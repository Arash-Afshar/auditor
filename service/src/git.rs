use std::collections::HashMap;

use crate::{Diff, LineDiff, MyError};
use git2::{Oid, Patch, Repository, Tree};

pub struct Git {
    repo: Repository,
}

impl Git {
    pub fn new(path: String) -> Result<Self, MyError> {
        let repo = Repository::open(path)?;
        Ok(Git { repo })
    }

    pub fn current_commit(&self) -> String {
        let commit = self.repo.head().unwrap().peel_to_commit().unwrap();
        commit.id().to_string()
    }

    pub fn get_tree_from_commit(&self, commit: &str) -> Result<Tree, MyError> {
        let commit = Oid::from_str(&commit)?;
        let commit = self.repo.find_commit(commit)?;
        let tree = commit.tree()?;
        Ok(tree)
    }

    pub fn diff_current_and_commit(
        &self,
        old_commit: Option<String>,
        exclusions: &Vec<String>,
    ) -> Result<Option<Diff>, MyError> {
        if old_commit.is_none() {
            return Ok(None);
        }
        let old_tree = self.get_tree_from_commit(&old_commit.unwrap())?;
        let current_tree = self.get_tree_from_commit(&self.current_commit())?;
        let diff = self
            .repo
            .diff_tree_to_tree(Some(&old_tree), Some(&current_tree), None)?;

        let mut files: HashMap<String, Vec<LineDiff>> = HashMap::default();
        if diff.deltas().len() == 0 {
            return Ok(None);
        }
        for (delta_index, delta) in diff.deltas().enumerate() {
            // TODO: what if a file is moved?
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
            if diff_content.is_some() {
                let diff_content = diff_content.unwrap();
                for hunk_index in 0..diff_content.num_hunks() {
                    let hunk_line_count = diff_content.num_lines_in_hunk(hunk_index)?;
                    for line_index in 0..hunk_line_count {
                        let diff_line = diff_content.line_in_hunk(hunk_index, line_index).unwrap();
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
        let git = Git::new("..".to_string()).unwrap();
        let current = git.current_commit();
        let prev = "700610cb3aa6b28dde21e854ba4547e29b766a48".to_string();
        // assert_eq!(current, "c9085e7d80b737d25c3986fa55c8968d48ce8898");
        let diff = git
            .diff_current_and_commit(Some(prev), &vec!["service".to_string()])
            .unwrap();
        dbg!(diff);
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
