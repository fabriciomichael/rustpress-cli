use std::io;

const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while",
];

const WINDOWS_RESERVED: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

pub fn validate_project_name(name: &str) -> io::Result<()> {
    if name.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The project name cannot be empty.",
        ));
    }

    if name.len() > 64 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The project name cannot exceed 64 characters.",
        ));
    }

    let mut chars = name.chars();
    let first = chars.next().unwrap();

    if !first.is_ascii_alphabetic() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The project name must start with a letter.",
        ));
    }

    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The name can only contain letters, numbers, hyphens, and underscores.",
        ));
    }

    if RUST_KEYWORDS.contains(&name) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The project name cannot be a Rust keyword.",
        ));
    }

    if WINDOWS_RESERVED.contains(&name.to_ascii_uppercase().as_str()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The project name is reserved by Windows.",
        ));
    }

    Ok(())
}

pub fn validate_module_name(name: &str) -> io::Result<()> {
    if name.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The module name cannot be empty.",
        ));
    }

    if name.len() > 64 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The module name cannot exceed 64 characters.",
        ));
    }

    let mut chars = name.chars();
    let first = chars.next().unwrap();

    if !first.is_ascii_lowercase() && first != '_' {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The module name must start with a lowercase letter or underscore.",
        ));
    }

    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The module name can only contain lowercase letters, numbers, and underscores.",
        ));
    }

    if RUST_KEYWORDS.contains(&name) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The module name cannot be a Rust keyword.",
        ));
    }

    Ok(())
}

pub fn normalize_module_name(name: &str) -> io::Result<String> {
    if name.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The module name cannot be empty.",
        ));
    }

    let chars: Vec<char> = name.chars().collect();
    let mut normalized = String::new();

    for (index, &c) in chars.iter().enumerate() {
        if c.is_ascii_uppercase() {
            let previous = index.checked_sub(1).map(|i| chars[i]);
            let next = chars.get(index + 1).copied();

            let needs_separator = previous.is_some_and(|prev| {
                prev != '_'
                    && (prev.is_ascii_lowercase()
                        || prev.is_ascii_digit()
                        || (prev.is_ascii_uppercase()
                            && next.is_some_and(|next| next.is_ascii_lowercase())))
            });

            if needs_separator {
                normalized.push('_');
            }

            normalized.push(c.to_ascii_lowercase());
        } else if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' {
            normalized.push(c);
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "The module name can only contain letters, numbers, and underscores.",
            ));
        }
    }

    validate_module_name(&normalized)?;

    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_project_names() {
        assert!(validate_project_name("minha_app").is_ok());
        assert!(validate_project_name("minha-app").is_ok());
        assert!(validate_project_name("gestao-escolar-2026").is_ok());
    }

    #[test]
    fn rejects_invalid_project_names() {
        assert!(validate_project_name("").is_err());
        assert!(validate_project_name("123app").is_err());
        assert!(validate_project_name("minha app").is_err());
        assert!(validate_project_name("minha.app").is_err());
        assert!(validate_project_name("fn").is_err());
        assert!(validate_project_name("CON").is_err());
        assert!(validate_project_name("COM1").is_err());
    }

    #[test]
    fn rejects_names_longer_than_64_characters() {
        let name = "a".repeat(65);

        assert!(validate_project_name(&name).is_err());
    }

    #[test]
    fn accepts_valid_module_names() {
        assert!(validate_module_name("user_controller").is_ok());
        assert!(validate_module_name("auth").is_ok());
        assert!(validate_module_name("_internal").is_ok());
        assert!(validate_module_name("controller2026").is_ok());
    }

    #[test]
    fn rejects_invalid_module_names() {
        assert!(validate_module_name("").is_err());
        assert!(validate_module_name("UserController").is_err());
        assert!(validate_module_name("user-controller").is_err());
        assert!(validate_module_name("123controller").is_err());
        assert!(validate_module_name("user controller").is_err());
        assert!(validate_module_name("fn").is_err());
    }

    #[test]
    fn normalizes_module_names() {
        assert_eq!(
            normalize_module_name("UserController").unwrap(),
            "user_controller"
        );

        assert_eq!(
            normalize_module_name("AuthMiddleware").unwrap(),
            "auth_middleware"
        );

        assert_eq!(
            normalize_module_name("EmailService").unwrap(),
            "email_service"
        );

        assert_eq!(normalize_module_name("User").unwrap(), "user");

        assert_eq!(
            normalize_module_name("user_controller").unwrap(),
            "user_controller"
        );

        assert_eq!(
            normalize_module_name("User_Controller").unwrap(),
            "user_controller"
        );

        assert_eq!(normalize_module_name("HTTPServer").unwrap(), "http_server");
    }

    #[test]
    fn rejects_invalid_module_names_during_normalization() {
        assert!(normalize_module_name("").is_err());
        assert!(normalize_module_name("user-controller").is_err());
        assert!(normalize_module_name("user controller").is_err());
        assert!(normalize_module_name("user.controller").is_err());
    }
}
