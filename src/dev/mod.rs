use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::mpsc::{self, Receiver};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// Determines whether a filesystem event belongs to a relevant project path.
fn is_relevant_event(project_path: &Path, event: &Event) -> bool {
    event.paths.iter().any(|path| {
        let Ok(relative_path) = path.strip_prefix(project_path) else {
            return false;
        };

        let ignored_directories = ["target", "dist", ".git"];

        !relative_path.components().any(|component| {
            ignored_directories.contains(&component.as_os_str().to_string_lossy().as_ref())
        })
    })
}

/// Creates a file watcher for the specified project directory.
pub fn create_watcher(
    project_path: &Path,
) -> Result<(RecommendedWatcher, Receiver<notify::Result<Event>>), String> {
    let project_path = project_path
        .canonicalize()
        .map_err(|error| format!("Failed to resolve project directory: {}", error))?;

    let (sender, receiver) = mpsc::channel();

    let watched_path = project_path.clone();

    let mut watcher = RecommendedWatcher::new(
        move |event| match event {
            Ok(event) if is_relevant_event(&watched_path, &event) => {
                let _ = sender.send(Ok(event));
            }
            Err(error) => {
                let _ = sender.send(Err(error));
            }
            _ => {}
        },
        Config::default(),
    )
    .map_err(|error| format!("Failed to create file watcher: {}", error))?;

    watcher
        .watch(&project_path, RecursiveMode::Recursive)
        .map_err(|error| format!("Failed to watch project directory: {}", error))?;

    Ok((watcher, receiver))
}

/// Waits for filesystem changes and groups events within a 500 ms debounce period.
pub fn wait_for_change(
    receiver: &Receiver<notify::Result<Event>>,
    shutdown: &AtomicBool,
) -> Result<bool, String> {
    use std::time::{Duration, Instant};

    let poll_interval = Duration::from_millis(100);
    let debounce_period = Duration::from_millis(500);

    // Wait for the first filesystem event.
    loop {
        if shutdown.load(Ordering::SeqCst) {
            return Ok(false);
        }

        match receiver.recv_timeout(poll_interval) {
            Ok(Ok(_event)) => break,
            Ok(Err(error)) => {
                return Err(format!("Filesystem watcher error: {}", error));
            }
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err("Filesystem watcher channel disconnected".to_string());
            }
        }
    }

    // Reset the debounce deadline whenever another event arrives.
    let mut deadline = Instant::now() + debounce_period;

    loop {
        if shutdown.load(Ordering::SeqCst) {
            return Ok(false);
        }

        let remaining = deadline.saturating_duration_since(Instant::now());

        if remaining.is_zero() {
            return Ok(true);
        }

        match receiver.recv_timeout(remaining.min(poll_interval)) {
            Ok(Ok(_event)) => {
                deadline = Instant::now() + debounce_period;
            }
            Ok(Err(error)) => {
                return Err(format!("Filesystem watcher error: {}", error));
            }
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err("Filesystem watcher channel disconnected".to_string());
            }
        }
    }
}

/// Finds the main executable produced by the Cargo project.
pub fn find_binary(project_dir: &Path) -> Result<PathBuf, String> {
    crate::cargo_metadata::find_binary(project_dir, "debug")
}

/// Builds the project and returns the generated executable path.
pub fn build_project(project_dir: &Path) -> Result<PathBuf, String> {
    println!("Building RustPress project...");

    let output = Command::new("cargo")
        .arg("build")
        .current_dir(project_dir)
        .output()
        .map_err(|error| format!("Failed to execute cargo build: {}", error))?;

    if !output.status.success() {
        return Err(format!(
            "Project build failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let binary_path = find_binary(project_dir)?;

    if !binary_path.exists() {
        return Err(format!(
            "Build completed, but executable was not found: {}",
            binary_path.display()
        ));
    }

    println!("Build completed successfully.");

    Ok(binary_path)
}

/// Starts the compiled application as a child process.
pub fn start_server(binary_path: &Path) -> Result<Child, String> {
    if !binary_path.exists() {
        return Err(format!("Executable not found: {}", binary_path.display()));
    }

    println!("Starting RustPress application...");

    Command::new(binary_path)
        .spawn()
        .map_err(|error| format!("Failed to start RustPress application: {}", error))
}

/// Stops the running application process.
pub fn stop_server(child: &mut Child) -> Result<(), String> {
    if child
        .try_wait()
        .map_err(|error| format!("Failed to check server process: {}", error))?
        .is_some()
    {
        return Ok(());
    }

    child
        .kill()
        .map_err(|error| format!("Failed to stop server process: {}", error))?;

    child
        .wait()
        .map_err(|error| format!("Failed to wait for server process: {}", error))?;

    println!("RustPress application stopped.");

    Ok(())
}

/// Registers a handler for Ctrl+C and returns a shared shutdown signal.
pub fn setup_shutdown_handler() -> Result<Arc<AtomicBool>, String> {
    let shutdown = Arc::new(AtomicBool::new(false));
    let signal = Arc::clone(&shutdown);

    ctrlc::set_handler(move || {
        signal.store(true, Ordering::SeqCst);
    })
    .map_err(|error| format!("Failed to register Ctrl+C handler: {}", error))?;

    Ok(shutdown)
}

/// Runs the project in development mode with automatic server restarts.
pub fn run_dev(project_dir: &Path) -> Result<(), String> {
    let project_dir = project_dir
        .canonicalize()
        .map_err(|error| format!("Failed to resolve project directory: {}", error))?;

    if !project_dir.join("Cargo.toml").exists() {
        return Err("No Cargo.toml found in the project directory".to_string());
    }

    // Create the watcher before the initial build to avoid missing changes.
    let (_watcher, receiver) = create_watcher(&project_dir)?;

    // Register Ctrl+C handling.
    let shutdown = setup_shutdown_handler()?;

    println!("Starting RustPress development mode...");
    println!("Watching: {}", project_dir.display());

    // Build and start the application.
    let binary_path = build_project(&project_dir)?;
    let mut server = Some(start_server(&binary_path)?);

    println!("Development server is running.");
    println!("Watching for changes. Press Ctrl+C to stop.");

    let result = loop {
        match wait_for_change(&receiver, &shutdown) {
            Ok(false) => {
                println!("\nShutting down development server...");
                break Ok(());
            }

            Err(error) => {
                break Err(error);
            }

            Ok(true) => {
                println!("\nChanges detected. Restarting application...");

                // Stop the previous process before rebuilding.
                if let Some(ref mut child) = server {
                    if let Err(error) = stop_server(child) {
                        break Err(error);
                    }
                }

                server = None;

                // A failed rebuild should not terminate development mode.
                let binary_path = match build_project(&project_dir) {
                    Ok(path) => path,
                    Err(error) => {
                        eprintln!("{}", error);
                        println!("Waiting for the next file change...");
                        continue;
                    }
                };

                match start_server(&binary_path) {
                    Ok(child) => {
                        server = Some(child);
                        println!("Development server restarted.");
                    }
                    Err(error) => {
                        eprintln!("{}", error);
                        println!("Waiting for the next file change...");
                    }
                }
            }
        }
    };

    // Ensure the application process is stopped before returning.
    if let Some(ref mut child) = server {
        stop_server(child)?;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn detects_file_creation() {
        let temp_dir =
            std::env::temp_dir().join(format!("rustpress-watcher-test-{}", std::process::id()));

        fs::create_dir_all(&temp_dir).unwrap();

        let (_watcher, receiver) = create_watcher(&temp_dir).unwrap();

        thread::sleep(Duration::from_millis(300));

        fs::write(temp_dir.join("test.rs"), "fn main() {}").unwrap();

        let event = receiver.recv_timeout(Duration::from_secs(5));

        assert!(event.is_ok(), "File creation event was not detected");

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn ignores_build_and_git_directories() {
        use notify::Event;

        let project_path =
            std::env::temp_dir().join(format!("rustpress-filter-test-{}", std::process::id()));

        fs::create_dir_all(&project_path).unwrap();

        let ignored_paths = [
            project_path.join("target").join("debug").join("app.exe"),
            project_path.join("dist").join("app.js"),
            project_path.join(".git").join("config"),
        ];

        for path in ignored_paths {
            let event = Event::new(notify::EventKind::Create(notify::event::CreateKind::File))
                .add_path(path);

            assert!(
                !is_relevant_event(&project_path, &event),
                "An ignored directory was incorrectly accepted"
            );
        }

        let relevant_event = Event::new(notify::EventKind::Create(notify::event::CreateKind::File))
            .add_path(project_path.join("src").join("main.rs"));

        assert!(is_relevant_event(&project_path, &relevant_event));

        fs::remove_dir_all(&project_path).unwrap();
    }

    #[test]
    fn groups_multiple_events_with_debounce() {
        use std::sync::mpsc;
        use std::thread;
        use std::time::Instant;

        let (sender, receiver) = mpsc::channel();
        let shutdown = AtomicBool::new(false);

        for _ in 0..3 {
            sender.send(Ok(Event::new(notify::EventKind::Any))).unwrap();
        }

        let start = Instant::now();

        let changed = wait_for_change(&receiver, &shutdown).unwrap();

        assert!(changed, "Expected filesystem changes");

        assert!(
            start.elapsed() >= Duration::from_millis(450),
            "Debounce period was shorter than expected"
        );

        assert!(
            matches!(receiver.try_recv(), Err(mpsc::TryRecvError::Empty)),
            "Pending events were not consumed"
        );

        thread::sleep(Duration::from_millis(10));
    }

    #[test]
    fn finds_project_binary() {
        let project_dir = std::env::current_dir().unwrap();

        let binary_path = find_binary(&project_dir).unwrap();

        assert_eq!(
            binary_path.file_stem().unwrap().to_string_lossy(),
            "rustpress"
        );

        assert!(
            binary_path.ends_with(Path::new("target").join("debug").join(if cfg!(windows) {
                "rustpress.exe"
            } else {
                "rustpress"
            }))
        );
    }

    #[test]
    fn builds_project_successfully() {
        let project_dir = std::env::current_dir().unwrap();

        let binary_path = build_project(&project_dir).unwrap();

        assert!(
            binary_path.exists(),
            "The generated executable does not exist"
        );
    }

    #[test]
    fn stops_running_process() {
        #[cfg(windows)]
        let mut child = Command::new("cmd")
            .args(["/C", "ping", "-n", "10", "127.0.0.1"])
            .spawn()
            .unwrap();

        #[cfg(unix)]
        let mut child = Command::new("sh").args(["-c", "sleep 10"]).spawn().unwrap();

        stop_server(&mut child).unwrap();

        assert!(
            child.try_wait().unwrap().is_some(),
            "Server process is still running"
        );
    }
}
