use std::collections::BTreeSet;
use std::env;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

pub const CLAUDE_CONFIG_DIR_ENV: &str = "CLAUDE_CONFIG_DIR";

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataRootOptions {
    pub explicit_project_roots: Vec<PathBuf>,
    pub claude_config_dir: Option<OsString>,
    pub home_dir: Option<PathBuf>,
}

impl DataRootOptions {
    #[must_use]
    pub fn from_environment() -> Self {
        Self {
            explicit_project_roots: Vec::new(),
            claude_config_dir: env::var_os(CLAUDE_CONFIG_DIR_ENV),
            home_dir: resolve_home_dir(),
        }
    }

    #[must_use]
    pub fn resolve_project_roots(&self) -> Vec<PathBuf> {
        if !self.explicit_project_roots.is_empty() {
            return dedupe_and_sort(self.explicit_project_roots.clone());
        }

        let home_dir = self.effective_home_dir();
        if let Some(raw_roots) = self.claude_config_dir.as_deref() {
            let parsed = parse_claude_config_dir(raw_roots, home_dir.as_deref());
            if !parsed.is_empty() {
                return parsed;
            }
        }

        self.default_project_roots(home_dir.as_deref())
    }

    fn effective_home_dir(&self) -> Option<PathBuf> {
        self.home_dir.clone().or_else(resolve_home_dir)
    }

    fn default_project_roots(&self, home_dir: Option<&Path>) -> Vec<PathBuf> {
        let Some(home_dir) = home_dir else {
            return Vec::new();
        };

        dedupe_and_sort(vec![
            home_dir.join(".config").join("claude").join("projects"),
            home_dir.join(".claude").join("projects"),
        ])
    }
}

#[must_use]
pub fn resolve_home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
        .or_else(|| {
            let home_drive = env::var_os("HOMEDRIVE")?;
            let home_path = env::var_os("HOMEPATH")?;
            let mut combined = PathBuf::from(home_drive);
            combined.push(home_path);
            Some(combined)
        })
}

fn parse_claude_config_dir(raw: &OsStr, home_dir: Option<&Path>) -> Vec<PathBuf> {
    let raw_text = raw.to_string_lossy();
    let config_roots = raw_text
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(|entry| expand_home_dir(entry, home_dir));

    let mut project_roots = Vec::new();
    for config_root in config_roots {
        project_roots.extend(config_root_to_projects(&config_root));
    }

    dedupe_and_sort(project_roots)
}

fn expand_home_dir(entry: &str, home_dir: Option<&Path>) -> PathBuf {
    if entry == "~" {
        return home_dir
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from(entry));
    }

    if let Some(stripped) = entry.strip_prefix("~/")
        && let Some(home_dir) = home_dir
    {
        return home_dir.join(stripped);
    }

    PathBuf::from(entry)
}

fn config_root_to_projects(config_root: &Path) -> Vec<PathBuf> {
    if config_root.as_os_str().is_empty() {
        return Vec::new();
    }

    if config_root
        .file_name()
        .is_some_and(|name| name == OsStr::new("projects"))
    {
        return vec![config_root.to_path_buf()];
    }

    vec![config_root.join("projects")]
}

fn dedupe_and_sort(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut deduped = BTreeSet::new();
    for path in paths {
        if path.as_os_str().is_empty() {
            continue;
        }
        deduped.insert(make_absolute(path));
    }
    deduped.into_iter().collect()
}

fn make_absolute(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        return path;
    }

    match env::current_dir() {
        Ok(current_dir) => current_dir.join(path),
        Err(_) => path,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_default_roots_when_no_overrides_exist() {
        let options = DataRootOptions {
            explicit_project_roots: Vec::new(),
            claude_config_dir: None,
            home_dir: Some(PathBuf::from("/home/tester")),
        };

        let roots = options.resolve_project_roots();

        assert_eq!(
            roots,
            vec![
                PathBuf::from("/home/tester/.claude/projects"),
                PathBuf::from("/home/tester/.config/claude/projects"),
            ]
        );
    }

    #[test]
    fn explicit_project_roots_override_env_and_defaults() {
        let options = DataRootOptions {
            explicit_project_roots: vec![
                PathBuf::from("fixtures/custom-root"),
                PathBuf::from("fixtures/custom-root"),
            ],
            claude_config_dir: Some(OsString::from("/ignored")),
            home_dir: Some(PathBuf::from("/home/tester")),
        };

        let roots = options.resolve_project_roots();

        assert_eq!(roots.len(), 1);
        assert!(roots[0].is_absolute());
        assert!(roots[0].ends_with("fixtures/custom-root"));
    }

    #[test]
    fn claude_config_dir_supports_multiple_roots_and_projects_suffix() {
        let options = DataRootOptions {
            explicit_project_roots: Vec::new(),
            claude_config_dir: Some(OsString::from(
                "~/.claude-alt,/tmp/claude-a,/tmp/claude-b/projects",
            )),
            home_dir: Some(PathBuf::from("/home/tester")),
        };

        let roots = options.resolve_project_roots();

        assert_eq!(
            roots,
            vec![
                PathBuf::from("/home/tester/.claude-alt/projects"),
                PathBuf::from("/tmp/claude-a/projects"),
                PathBuf::from("/tmp/claude-b/projects"),
            ]
        );
    }

    #[test]
    fn empty_claude_config_dir_entries_fall_back_to_default_roots() {
        let options = DataRootOptions {
            explicit_project_roots: Vec::new(),
            claude_config_dir: Some(OsString::from(" , , ")),
            home_dir: Some(PathBuf::from("/home/tester")),
        };

        let roots = options.resolve_project_roots();

        assert_eq!(
            roots,
            vec![
                PathBuf::from("/home/tester/.claude/projects"),
                PathBuf::from("/home/tester/.config/claude/projects"),
            ]
        );
    }
}
