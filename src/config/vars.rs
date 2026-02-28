use color_eyre::Result;
use std::collections::HashMap;

/// Resolve template variables in a string.
/// Replaces `{{var_name}}` with the corresponding value.
pub fn resolve_vars(input: &str, vars: &HashMap<String, String>) -> String {
    let mut result = input.to_string();
    for (key, value) in vars {
        let pattern = format!("{{{{{}}}}}", key);
        result = result.replace(&pattern, value);
    }
    result
}

/// Build the default variable map from system environment.
pub fn build_default_vars() -> Result<HashMap<String, String>> {
    let mut vars = HashMap::new();

    // Home directory
    if let Some(home) = dirs::home_dir() {
        vars.insert("home".to_string(), home.to_string_lossy().to_string());
    }

    // Windows-specific directories
    if let Ok(appdata) = std::env::var("APPDATA") {
        vars.insert("appdata".to_string(), appdata);
    }

    if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
        vars.insert("localappdata".to_string(), localappdata);
    }

    // Documents folder
    if let Some(home) = dirs::home_dir() {
        let docs = home.join("Documents");
        vars.insert("docs".to_string(), docs.to_string_lossy().to_string());
    }

    // Repo root (current working directory or detected via git)
    if let Ok(cwd) = std::env::current_dir() {
        vars.insert("repo_root".to_string(), cwd.to_string_lossy().to_string());
    }

    Ok(vars)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_vars() {
        let mut vars = HashMap::new();
        vars.insert("home".to_string(), "C:\\Users\\test".to_string());
        vars.insert(
            "bin_dir".to_string(),
            "C:\\Users\\test\\.config\\bin".to_string(),
        );

        assert_eq!(
            resolve_vars("{{home}}/.config/starship.toml", &vars),
            "C:\\Users\\test/.config/starship.toml"
        );
        assert_eq!(
            resolve_vars("{{bin_dir}}/flatten.exe", &vars),
            "C:\\Users\\test\\.config\\bin/flatten.exe"
        );
    }

    #[test]
    fn test_no_vars() {
        let vars = HashMap::new();
        assert_eq!(resolve_vars("plain string", &vars), "plain string");
    }
}
