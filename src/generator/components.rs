use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use super::validation;

fn component_template(name: &str) -> &'static str {
    match name {
        "button" => {
            r#"<button type="button" class="btn">
    Button
</button>
"#
        }

        "input" => {
            r#"<div class="form-group">
    <label for="input">Label</label>
    <input type="text" id="input" name="input" class="form-control" placeholder="Enter text">
</div>
"#
        }

        "card" => {
            r#"<article class="card">
    <div class="card-body">
        <h2 class="card-title">Card title</h2>
        <p class="card-text">Card content goes here.</p>
    </div>
</article>
"#
        }

        "alert" => {
            r#"<div class="alert" role="alert">
    Alert message.
</div>
"#
        }

        "badge" => {
            r#"<span class="badge">Badge</span>
"#
        }

        _ => "<!-- Custom component -->\n",
    }
}

pub fn create_component(project_path: &Path, name: &str) -> io::Result<String> {
    let component_name = validation::normalize_module_name(name)?;

    let cargo_file = project_path.join("Cargo.toml");
    let views_dir = project_path.join("views");
    let components_dir = views_dir.join("components");

    if !cargo_file.is_file() || !views_dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The current directory is not a valid RustPress project.",
        ));
    }

    fs::create_dir_all(&components_dir)?;

    let component_file = components_dir.join(format!("{component_name}.html"));

    let template = component_template(&component_name);

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&component_file)?;

    if let Err(error) = file.write_all(template.as_bytes()) {
        drop(file);
        let _ = fs::remove_file(&component_file);
        return Err(error);
    }

    Ok(component_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_project() -> std::path::PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let project = std::env::temp_dir().join(format!("rustpress-components-test-{timestamp}"));

        fs::create_dir_all(project.join("views")).unwrap();

        fs::write(
            project.join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        project
    }

    #[test]
    fn creates_component_file() {
        let project = temporary_project();

        let result = create_component(&project, "button");

        assert_eq!(result.unwrap(), "button");

        let component_file = project.join("views/components/button.html");

        assert!(component_file.is_file());

        let content = fs::read_to_string(&component_file).unwrap();

        assert_eq!(
            content,
            "<button type=\"button\" class=\"btn\">\n    Button\n</button>\n"
        );

        fs::remove_dir_all(project).unwrap();
    }

    #[test]
    fn creates_component_from_pascal_case_name() {
        let project = temporary_project();

        let result = create_component(&project, "ProductCard");

        assert_eq!(result.unwrap(), "product_card");

        assert!(project.join("views/components/product_card.html").is_file());

        fs::remove_dir_all(project).unwrap();
    }

    #[test]
    fn rejects_duplicate_component_without_overwriting() {
        let project = temporary_project();

        let components_dir = project.join("views/components");
        fs::create_dir_all(&components_dir).unwrap();

        let component_file = components_dir.join("button.html");
        let original_content = "<!-- Existing button -->\n";

        fs::write(&component_file, original_content).unwrap();

        let result = create_component(&project, "button");

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::AlreadyExists);

        assert_eq!(
            fs::read_to_string(component_file).unwrap(),
            original_content
        );

        fs::remove_dir_all(project).unwrap();
    }

    #[test]
    fn rejects_invalid_rustpress_project() {
        let project = std::env::temp_dir().join("rustpress-invalid-components-project");

        let result = create_component(&project, "button");

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn rejects_invalid_component_name() {
        let project = temporary_project();

        let result = create_component(&project, "123-button");

        assert!(result.is_err());

        assert!(!project.join("views/components").exists());

        fs::remove_dir_all(project).unwrap();
    }

    #[test]
    fn creates_input_component() {
        let project = temporary_project();

        create_component(&project, "input").unwrap();

        let content = fs::read_to_string(project.join("views/components/input.html")).unwrap();

        assert!(content.contains("<input type=\"text\""));
        assert!(content.contains("<label"));

        fs::remove_dir_all(project).unwrap();
    }

    #[test]
    fn creates_card_component() {
        let project = temporary_project();

        create_component(&project, "card").unwrap();

        let content = fs::read_to_string(project.join("views/components/card.html")).unwrap();

        assert!(content.contains("<article class=\"card\">"));
        assert!(content.contains("card-title"));

        fs::remove_dir_all(project).unwrap();
    }

    #[test]
    fn creates_alert_component() {
        let project = temporary_project();

        create_component(&project, "alert").unwrap();

        let content = fs::read_to_string(project.join("views/components/alert.html")).unwrap();

        assert!(content.contains("role=\"alert\""));
        assert!(content.contains("Alert message."));

        fs::remove_dir_all(project).unwrap();
    }

    #[test]
    fn creates_badge_component() {
        let project = temporary_project();

        create_component(&project, "badge").unwrap();

        let content = fs::read_to_string(project.join("views/components/badge.html")).unwrap();

        assert!(content.contains("<span class=\"badge\">"));

        fs::remove_dir_all(project).unwrap();
    }
}
