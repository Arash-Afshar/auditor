use crate::{AuditorError, Diff, LineDiff};
use git2::{Oid, Patch, Repository, Tree};
use std::collections::HashMap;

pub struct Git {
    repo: Repository,
}

impl Git {
    pub fn new(path: &String) -> Result<Self, AuditorError> {
        let repo = Repository::open(path)?;
        Ok(Git { repo })
    }

    pub fn current_commit(&self) -> Result<String, AuditorError> {
        let commit = self.repo.head()?.peel_to_commit()?;
        Ok(commit.id().to_string())
    }

    pub fn get_tree_from_commit(&self, commit: &str) -> Result<Tree, AuditorError> {
        let commit = Oid::from_str(commit)?;
        let commit = self.repo.find_commit(commit)?;
        let tree = commit.tree()?;
        Ok(tree)
    }

    pub fn diff_current_and_commit(
        &self,
        old_commit: Option<String>,
        exclusions: &Vec<String>,
    ) -> Result<Option<Diff>, AuditorError> {
        if old_commit.is_none() {
            return Ok(None);
        }
        let old_commit = old_commit.expect("will never fail");
        let old_tree = self.get_tree_from_commit(&old_commit)?;
        let current_tree = self.get_tree_from_commit(&self.current_commit()?)?;
        let diff = self
            .repo
            .diff_tree_to_tree(Some(&old_tree), Some(&current_tree), None)?;

        let mut files: HashMap<String, Vec<LineDiff>> = HashMap::default();
        if diff.deltas().len() == 0 {
            return Ok(None);
        }
        for (delta_index, delta) in diff.deltas().enumerate() {
            // TODO: what if a file is moved?
            // TODO: handle the unwraps properly
            let old_file_name = delta
                .old_file()
                .path()
                .unwrap()
                .as_os_str()
                .to_str()
                .unwrap();
            let mut in_exclusion = false;
            for prefix in exclusions {
                if old_file_name.starts_with(prefix) {
                    in_exclusion = true;
                    break;
                }
            }
            if in_exclusion {
                continue;
            }
            let diff_content = Patch::from_diff(&diff, delta_index)?;
            let mut line_diffs = vec![];
            if let Some(diff_content) = diff_content {
                for hunk_index in 0..diff_content.num_hunks() {
                    let hunk_line_count = diff_content.num_lines_in_hunk(hunk_index)?;
                    for line_index in 0..hunk_line_count {
                        let diff_line = diff_content.line_in_hunk(hunk_index, line_index)?;
                        let old = diff_line.old_lineno();
                        let new = diff_line.new_lineno();
                        let is_modified = old.is_none() || new.is_none();
                        if is_modified {
                            line_diffs.push(LineDiff { old, new });
                        }
                    }
                }
            }
            files.insert(old_file_name.to_string(), line_diffs);
        }

        Ok(Some(Diff { files }))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_diff() {
        let git = Git::new(&"..".to_string()).unwrap();
        let current = git.current_commit().unwrap();
        let prev = "700610cb3aa6b28dde21e854ba4547e29b766a48".to_string();
        // assert_eq!(current, "c9085e7d80b737d25c3986fa55c8968d48ce8898");
        let _diff = git
            .diff_current_and_commit(Some(prev), &vec!["service".to_string()])
            .unwrap();
        assert!(git
            .diff_current_and_commit(None, &vec![])
            .unwrap()
            .is_none());
        assert!(git
            .diff_current_and_commit(Some(current), &vec![])
            .unwrap()
            .is_none());
    }
}
