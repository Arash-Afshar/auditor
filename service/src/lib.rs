use db::DB;
use git::Git;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};
pub mod db;
pub mod git;

// TODO: rewrite with thiserror
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

#[derive(Deserialize, Debug)]
pub enum State {
    Reviewed,
    Modified,
    Cleared,
}

#[derive(Deserialize, Debug)]
pub struct UpdateReviewState {
    file_name: String,
    _start_line: usize,
    _end_line: usize,
    _review_state: State,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct StoredReviewForFile {
    pub reviewed: HashSet<usize>,
    pub modified: HashSet<usize>,
}

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Debug)]
pub struct LineDiff {
    old: Option<u32>,
    new: Option<u32>,
}

#[derive(Debug)]
pub struct Diff {
    files: HashMap<String, Vec<LineDiff>>,
}

pub fn get_review_state(
    file_name: String,
    db: &mut DB,
    git: &Git,
) -> Result<StoredReviewForFile, MyError> {
    let commit = db.latest_reviewed_commit(&file_name);
    let mut state = db.review_status_of_commit(&commit);
    let diff = git.diff_current_and_commit(commit, (state.exclusions).as_ref())?;
    if !diff.is_none() {
        state = transform_reviews(state, diff);
    }
    db.store_review_status(&git.current_commit(), &state)?;
    Ok(match state.files.get(&file_name) {
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
    if (&commit).is_none() || commit.as_ref().unwrap() != &git.current_commit() {
        return Err(MyError {
            message: "Call GET first".to_string(),
        });
    }

    let state = db.review_status_of_commit(&commit);
    let new_state = update_reviews(state, changes);
    db.store_review_status(&git.current_commit(), &new_state)?;
    return Ok(());
}

fn transform_reviews(
    current_state: StoredReviewForCommit,
    _diff: Option<Diff>,
) -> StoredReviewForCommit {
    current_state
}

fn update_reviews(
    current_state: StoredReviewForCommit,
    _changes: UpdateReviewState,
) -> StoredReviewForCommit {
    current_state
}
