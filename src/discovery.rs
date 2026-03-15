use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiscoveryResult {
    pub files: Vec<PathBuf>,
    pub warnings: Vec<DiscoveryWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryWarning {
    pub path: PathBuf,
    pub message: String,
}

#[must_use]
pub fn discover_session_files(roots: &[PathBuf]) -> DiscoveryResult {
    let unique_roots = dedupe_paths(roots.iter().cloned());
    let mut files = BTreeSet::new();
    let mut warnings = Vec::new();

    for root in unique_roots {
        discover_root(&root, &mut files, &mut warnings);
    }

    DiscoveryResult {
        files: files.into_iter().collect(),
        warnings,
    }
}

fn discover_root(root: &Path, files: &mut BTreeSet<PathBuf>, warnings: &mut Vec<DiscoveryWarning>) {
    let metadata = match fs::metadata(root) {
        Ok(metadata) => metadata,
        Err(error) => {
            warnings.push(DiscoveryWarning {
                path: root.to_path_buf(),
                message: format!("failed to stat root: {error}"),
            });
            return;
        }
    };

    if metadata.is_file() {
        if has_jsonl_extension(root) {
            files.insert(root.to_path_buf());
        }
        return;
    }

    if !metadata.is_dir() {
        return;
    }

    let mut stack = vec![root.to_path_buf()];
    while let Some(current_dir) = stack.pop() {
        let entries = match fs::read_dir(&current_dir) {
            Ok(entries) => entries,
            Err(error) => {
                warnings.push(DiscoveryWarning {
                    path: current_dir.clone(),
                    message: format!("failed to read directory: {error}"),
                });
                continue;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    warnings.push(DiscoveryWarning {
                        path: current_dir.clone(),
                        message: format!("failed to read directory entry: {error}"),
                    });
                    continue;
                }
            };

            let path = entry.path();
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(error) => {
                    warnings.push(DiscoveryWarning {
                        path: path.clone(),
                        message: format!("failed to inspect file type: {error}"),
                    });
                    continue;
                }
            };

            if file_type.is_dir() {
                stack.push(path);
            } else if file_type.is_file() && has_jsonl_extension(&path) {
                files.insert(path);
            }
        }
    }
}

fn has_jsonl_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("jsonl"))
}

fn dedupe_paths(paths: impl IntoIterator<Item = PathBuf>) -> Vec<PathBuf> {
    let mut unique = BTreeSet::new();
    for path in paths {
        if path.as_os_str().is_empty() {
            continue;
        }
        unique.insert(path);
    }
    unique.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir_all, remove_dir_all, write};
    use std::path::Path;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn discovers_jsonl_files_recursively_across_roots() {
        let test_dir = TestDir::new();
        let root_a = test_dir.path().join("a/projects");
        let root_b = test_dir.path().join("b/projects");

        create_dir_all(root_a.join("project-alpha")).expect("failed to create root a project dir");
        create_dir_all(root_b.join("project-beta/nested"))
            .expect("failed to create root b project dir");

        let file_a = root_a.join("project-alpha/session-1.jsonl");
        let file_b = root_b.join("project-beta/nested/session-2.JSONL");
        let ignored = root_b.join("project-beta/nested/README.md");
        write(&file_a, "{}\n").expect("failed to write session file a");
        write(&file_b, "{}\n").expect("failed to write session file b");
        write(&ignored, "# not usage data\n").expect("failed to write ignored file");

        let result = discover_session_files(&[root_b.clone(), root_a.clone(), root_a]);

        assert!(result.warnings.is_empty());
        assert_eq!(result.files, vec![file_a, file_b]);
    }

    #[test]
    fn accepts_file_roots_when_a_single_jsonl_file_is_targeted() {
        let test_dir = TestDir::new();
        let file_root = test_dir.path().join("single-session.jsonl");
        write(&file_root, "{}\n").expect("failed to write file root");

        let result = discover_session_files(std::slice::from_ref(&file_root));

        assert!(result.warnings.is_empty());
        assert_eq!(result.files, vec![file_root]);
    }

    #[test]
    fn records_warnings_for_missing_roots_without_failing_discovery() {
        let test_dir = TestDir::new();
        let existing_root = test_dir.path().join("projects");
        let missing_root = test_dir.path().join("missing");
        create_dir_all(&existing_root).expect("failed to create existing root");

        let session_file = existing_root.join("session.jsonl");
        write(&session_file, "{}\n").expect("failed to write session file");

        let result = discover_session_files(&[missing_root.clone(), existing_root]);

        assert_eq!(result.files, vec![session_file]);
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0].path, missing_root);
        assert!(result.warnings[0].message.contains("failed to stat root"));
    }

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let mut path = std::env::temp_dir();
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos();
            let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
            path.push(format!(
                "cusage-rs-discovery-tests-{}-{timestamp}-{counter}",
                std::process::id()
            ));

            create_dir_all(&path).expect("failed to create test directory");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = remove_dir_all(&self.path);
        }
    }
}
