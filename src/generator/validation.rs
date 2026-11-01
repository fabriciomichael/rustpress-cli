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
}
