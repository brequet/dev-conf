use serde::Deserialize;

/// Root configuration file (devconf.yaml)
#[derive(Debug, Deserialize, Clone)]
pub struct RootConfig {
    pub schema: u32,
    #[serde(default)]
    pub vars: std::collections::HashMap<String, String>,
}

/// A profile file (profiles/*.yaml)
#[derive(Debug, Deserialize, Clone)]
pub struct ProfileFile {
    pub profile: String,
    pub extends: Option<String>,
    #[serde(default)]
    pub packages: Vec<PackageEntry>,
    #[serde(default)]
    pub configs: Vec<ConfigEntry>,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub path: Vec<String>,
    #[serde(default)]
    pub actions: Vec<Action>,
}

/// Resolved profile after merging extends + remove
#[derive(Debug, Clone)]
pub struct ResolvedProfile {
    pub name: String,
    pub packages: Vec<Package>,
    pub configs: Vec<ConfigEntry>,
    pub env: std::collections::HashMap<String, String>,
    pub path: Vec<String>,
    pub actions: Vec<Action>,
}

/// A package entry as it appears in YAML (tagged union)
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum PackageEntry {
    Tagged(TaggedPackage),
}

/// Tagged package with source
#[derive(Debug, Deserialize, Clone)]
pub struct TaggedPackage {
    #[serde(default)]
    pub winget: Option<String>,
    #[serde(default)]
    pub scoop: Option<String>,
    #[serde(default)]
    pub choco: Option<String>,
    #[serde(default)]
    pub github: Option<GitHubPackage>,
    #[serde(default)]
    pub remove: Option<String>,
    #[serde(default)]
    pub before: Option<String>,
    #[serde(default)]
    pub after: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GitHubPackage {
    pub repo: String,
    pub asset: String,
    #[serde(rename = "as")]
    pub rename: Option<String>,
    pub to: Option<String>,
}

/// Normalized package after parsing
#[derive(Debug, Clone)]
pub struct Package {
    pub source: PackageSource,
    pub id: String,
    pub before: Option<String>,
    pub after: Option<String>,
}

#[derive(Debug, Clone)]
pub enum PackageSource {
    Winget,
    Scoop,
    Choco,
    GitHub(GitHubPackage),
}

/// Config file entry
#[derive(Debug, Deserialize, Clone)]
pub struct ConfigEntry {
    pub src: String,
    pub dest: String,
    pub method: ConfigMethod,
    #[serde(default)]
    pub mkdir: bool,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConfigMethod {
    Symlink,
    Copy,
}

/// Action to run (shell command)
#[derive(Debug, Deserialize, Clone)]
pub struct Action {
    pub name: String,
    pub shell: String,
    #[serde(default)]
    pub admin: bool,
}

/// Status of a package on the system
#[derive(Debug, Clone)]
pub enum PackageStatus {
    NotInstalled,
    Installed { version: String },
    Outdated { current: String, available: String },
    Unknown,
}

impl std::fmt::Display for PackageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageStatus::NotInstalled => write!(f, "not installed"),
            PackageStatus::Installed { version } => write!(f, "installed ({})", version),
            PackageStatus::Outdated { current, available } => {
                write!(f, "outdated ({} -> {})", current, available)
            }
            PackageStatus::Unknown => write!(f, "unknown"),
        }
    }
}
