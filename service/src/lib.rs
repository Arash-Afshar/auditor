use db::DB;
use git::Git;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};
pub mod db;
pub mod git;

// TODO: rewrite with thiserror and anyhow
#[derive(Debug)]
pub struct MyError {
    pub message: String,
}

impl<E: Display> From<E> for MyError {
    fn from(value: E) -> Self {
        MyError {
            message: value.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Comment {
    pub id: String,
    pub body: String,
    pub author: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileComments(pub HashMap<usize, Vec<Comment>>);

#[derive(Deserialize, Debug)]
pub enum State {
    Reviewed,
    Modified,
    Cleared,
}

#[derive(Deserialize, Debug)]
pub struct UpdateReviewState {
    pub file_name: String,
    start_line: usize,
    end_line: usize,
    review_state: State,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct StoredReviewForFile {
    pub reviewed: HashSet<usize>,
    pub modified: HashSet<usize>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StoredReviewForCommit {
    files: HashMap<String, StoredReviewForFile>,
    exclusions: Vec<String>,
}

impl StoredReviewForCommit {
    pub fn new(exclusions: Vec<String>) -> Self {
        Self {
            exclusions,
            files: HashMap::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LineDiff {
    old: Option<u32>,
    new: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Diff {
    files: HashMap<String, Vec<LineDiff>>,
}

pub fn get_review_state(
    file_name: &String,
    db: &mut DB,
    git: &Git,
) -> Result<StoredReviewForFile, MyError> {
    let commit = db.latest_reviewed_commit(&file_name);
    let mut state = db.review_status_of_commit(&commit);
    let diff = git.diff_current_and_commit(commit, (state.exclusions).as_ref())?;
    if diff.is_some() {
        state = transform_reviews(&state, diff);
        db.store_review_status(&git.current_commit(), &state)?;
    }
    Ok(match state.files.get(file_name) {
        Some(state) => state.clone(),
        None => StoredReviewForFile {
            reviewed: HashSet::default(),
            modified: HashSet::default(),
        },
    })
}

pub fn update_review_state(
    changes: UpdateReviewState,
    db: &mut DB,
    git: &Git,
) -> Result<(), MyError> {
    let commit = db.latest_reviewed_commit(&changes.file_name);
    let state = db.review_status_of_commit(&commit);
    let new_state = update_reviews(&state, changes);
    db.store_review_status(&git.current_commit(), &new_state)?;
    return Ok(());
}

fn transform_reviews(
    current_state: &StoredReviewForCommit,
    diff: Option<Diff>,
) -> StoredReviewForCommit {
    if diff.is_none() {
        return current_state.clone();
    }
    let diff = diff.unwrap();
    let mut new_state = current_state.clone();
    for (file_name, line_diffs) in diff.files {
        if !new_state.files.contains_key(&file_name) {
            new_state.files.insert(
                file_name.clone(),
                StoredReviewForFile {
                    reviewed: HashSet::default(),
                    modified: HashSet::default(),
                },
            );
        }
        let file_review = new_state.files.get_mut(&file_name).unwrap();
        for line_diff in line_diffs {
            if line_diff.new.is_some() {
                let new_line_number: usize = line_diff.new.unwrap().try_into().unwrap();
                let new_line_number = new_line_number - 1;
                file_review.reviewed.remove(&new_line_number);
                file_review.modified.insert(new_line_number);
            }
            if line_diff.old.is_some() && line_diff.new.is_none() {
                // TODO: for now going with a simplistic approach and ignoring the deleted lines.
                // A good portion of these cases are "modifications" meaning that a line was deleted and another line was added in its place.
                // println!("The old line {} is deleted.", line_diff.old.unwrap());
            }
        }
    }
    new_state
}

fn update_reviews(
    current_state: &StoredReviewForCommit,
    changes: UpdateReviewState,
) -> StoredReviewForCommit {
    let changed_lines = HashSet::from_iter(changes.start_line..changes.end_line + 1);
    let mut new_state = current_state.clone();
    let file_reviews = new_state.files.get_mut(&changes.file_name);
    match file_reviews {
        Some(current_file_reviews) => {
            match changes.review_state {
                State::Reviewed => {
                    current_file_reviews.reviewed.extend(&changed_lines);
                    for line in changed_lines {
                        current_file_reviews.modified.remove(&line);
                    }
                }
                State::Modified => {
                    current_file_reviews.modified.extend(&changed_lines);
                    for line in changed_lines {
                        current_file_reviews.reviewed.remove(&line);
                    }
                }
                State::Cleared => {
                    for line in &changed_lines {
                        current_file_reviews.reviewed.remove(line);
                    }
                    for line in changed_lines {
                        current_file_reviews.modified.remove(&line);
                    }
                }
            };
        }
        None => {
            let new_file_review = match changes.review_state {
                State::Reviewed => StoredReviewForFile {
                    reviewed: changed_lines,
                    modified: HashSet::default(),
                },
                State::Modified => StoredReviewForFile {
                    reviewed: HashSet::default(),
                    modified: changed_lines,
                },
                State::Cleared => StoredReviewForFile {
                    reviewed: HashSet::default(),
                    modified: HashSet::default(),
                },
            };
            new_state
                .files
                .insert(changes.file_name.clone(), new_file_review);
        }
    };
    new_state
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_transform_reviews() {
        let file1 = "file1".to_string();
        let file2 = "file2".to_string();
        let mut files: HashMap<String, StoredReviewForFile> = HashMap::default();
        files.insert(
            file1.clone(),
            StoredReviewForFile {
                reviewed: HashSet::from_iter(vec![0]),
                modified: HashSet::from_iter(vec![1]),
            },
        );
        let current_state = &StoredReviewForCommit {
            files,
            exclusions: vec![],
        };

        // ----------- test: Empty diff has no effect
        let diff = None;
        let updated_state = transform_reviews(current_state, diff);
        assert_eq!(current_state.files, updated_state.files);

        // ----------- test: Diff causes modified lines - init
        let mut diff_files = HashMap::default();
        diff_files.insert(
            file2.clone(),
            vec![LineDiff {
                old: None,
                new: Some(3),
            }],
        );
        let diff = Some(Diff { files: diff_files });
        let updated_state = transform_reviews(&updated_state, diff);
        assert_eq!(
            updated_state.files.get(&file1).unwrap().clone().reviewed,
            HashSet::from_iter(vec![0])
        );
        assert_eq!(
            updated_state.files.get(&file1).unwrap().clone().modified,
            HashSet::from_iter(vec![1])
        );
        assert_eq!(
            updated_state.files.get(&file2).unwrap().clone().reviewed,
            HashSet::default()
        );
        assert_eq!(
            updated_state.files.get(&file2).unwrap().clone().modified,
            HashSet::from_iter(vec![3])
        );

        // ----------- test: Diff causes modified lines - existing
        let mut diff_files = HashMap::default();
        diff_files.insert(
            file1.clone(),
            vec![LineDiff {
                old: None,
                new: Some(0),
            }],
        );
        let diff = Some(Diff { files: diff_files });
        let updated_state = transform_reviews(&updated_state, diff);
        assert_eq!(
            updated_state.files.get(&file1).unwrap().clone().reviewed,
            HashSet::default()
        );
        assert_eq!(
            updated_state.files.get(&file1).unwrap().clone().modified,
            HashSet::from_iter(vec![0, 1])
        );
        assert_eq!(
            updated_state.files.get(&file2).unwrap().clone().reviewed,
            HashSet::default()
        );
        assert_eq!(
            updated_state.files.get(&file2).unwrap().clone().modified,
            HashSet::from_iter(vec![3])
        );
    }

    #[test]
    fn test_update_reviews() {
        let file1 = "file1".to_string();
        let mut files: HashMap<String, StoredReviewForFile> = HashMap::default();
        files.insert(
            file1.clone(),
            StoredReviewForFile {
                reviewed: HashSet::from_iter(vec![0]),
                modified: HashSet::from_iter(vec![1]),
            },
        );
        let current_state = &StoredReviewForCommit {
            files,
            exclusions: vec![],
        };
        // --------------- test initial
        let changes = UpdateReviewState {
            file_name: file1.clone(),
            start_line: 3,
            end_line: 5,
            review_state: State::Reviewed,
        };
        let updated_state = update_reviews(current_state, changes);
        assert_eq!(
            updated_state.files.get(&file1).unwrap().clone().reviewed,
            HashSet::from_iter(vec![0, 3, 4, 5])
        );
        assert_eq!(
            updated_state.files.get(&file1).unwrap().clone().modified,
            HashSet::from_iter(vec![1])
        );

        // --------------- test change to modified
        let changes = UpdateReviewState {
            file_name: file1.clone(),
            start_line: 2,
            end_line: 4,
            review_state: State::Modified,
        };
        let updated_state = update_reviews(&updated_state, changes);
        assert_eq!(
            updated_state.files.get(&file1).unwrap().clone().reviewed,
            HashSet::from_iter(vec![0, 5])
        );
        assert_eq!(
            updated_state.files.get(&file1).unwrap().clone().modified,
            HashSet::from_iter(1..5)
        );

        // --------------- test change to reviewed
        let changes = UpdateReviewState {
            file_name: file1.clone(),
            start_line: 2,
            end_line: 3,
            review_state: State::Reviewed,
        };
        let updated_state = update_reviews(&updated_state, changes);
        assert_eq!(
            updated_state.files.get(&file1).unwrap().clone().reviewed,
            HashSet::from_iter(vec![0, 2, 3, 5])
        );
        assert_eq!(
            updated_state.files.get(&file1).unwrap().clone().modified,
            HashSet::from_iter(vec![1, 4])
        );

        // --------------- test clear
        let changes = UpdateReviewState {
            file_name: file1.clone(),
            start_line: 1,
            end_line: 5,
            review_state: State::Cleared,
        };
        let updated_state = update_reviews(&updated_state, changes);
        assert_eq!(
            updated_state.files.get(&file1).unwrap().clone().reviewed,
            HashSet::from_iter(vec![0])
        );
        assert_eq!(
            updated_state.files.get(&file1).unwrap().clone().modified,
            HashSet::default()
        );
    }
}
