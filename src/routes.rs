use std::fs;
use std::io;
use std::path::Path;

fn extract_routes(content: &str) -> Vec<(String, String)> {
    let mut routes = Vec::new();

    // Remove comentários de linha antes de analisar as chamadas.
    let source = content
        .lines()
        .map(|line| line.split_once("//").map_or(line, |(code, _)| code))
        .collect::<Vec<_>>()
        .join("\n");

    for method in ["get", "post", "put", "delete", "patch"] {
        let prefix = format!("app.{method}(");
        let mut remaining = source.as_str();

        while let Some(start) = remaining.find(&prefix) {
            let arguments = &remaining[start + prefix.len()..];

            if let Some(route) = arguments
                .trim_start()
                .strip_prefix('"')
                .and_then(|value| value.split('"').next())
            {
                routes.push((method.to_uppercase(), route.to_string()));
            }

            if arguments.is_empty() {
                break;
            }

            remaining = &arguments[1..];
        }
    }

    routes
}

pub fn list_routes(project_path: &Path) -> io::Result<()> {
    let routes_directory = project_path.join("src/routes");

    if !routes_directory.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The current directory is not a valid RustPress project.",
        ));
    }

    println!("RustPress routes:");

    for entry in fs::read_dir(routes_directory)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|value| value.to_str()) != Some("rs") {
            continue;
        }

        let content = fs::read_to_string(&path)?;

        for (method, route) in extract_routes(&content) {
            println!("{:<7} {}", method, route);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::extract_routes;

    #[test]
    fn extracts_single_line_routes() {
        let source = r#"
            app.get("/products", handler)?;
            app.post("/products", handler)?;
        "#;

        let routes = extract_routes(source);

        assert_eq!(routes.len(), 2);
        assert_eq!(routes[0], ("GET".to_string(), "/products".to_string()));
        assert_eq!(routes[1], ("POST".to_string(), "/products".to_string()));
    }

    #[test]
    fn extracts_multiline_routes() {
        let source = r#"
            app.get(
                "/products/:id",
                handler,
            )?;
        "#;

        let routes = extract_routes(source);

        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0], ("GET".to_string(), "/products/:id".to_string()));
    }

    #[test]
    fn extracts_all_supported_http_methods() {
        let source = r#"
            app.get("/get", handler)?;
            app.post("/post", handler)?;
            app.put("/put", handler)?;
            app.delete("/delete", handler)?;
            app.patch("/patch", handler)?;
        "#;

        let routes = extract_routes(source);

        assert_eq!(routes.len(), 5);
    }

    #[test]
    fn ignores_single_line_comments() {
        let source = r#"
            // app.get("/ignored", handler)?;
            app.get("/products", handler)?;
        "#;

        let routes = extract_routes(source);

        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].1, "/products");
    }

    #[test]
    fn handles_files_without_routes() {
        let source = "pub fn register() { Ok(()) }";

        let routes = extract_routes(source);

        assert!(routes.is_empty());
    }
}
