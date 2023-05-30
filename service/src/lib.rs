use db::DB;
use git::Git;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
mod db;
mod git;

pub struct Error {
    pub reason: String,
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
    start_line: usize,
    end_line: usize,
    review_state: State,
}

#[derive(Serialize, Deserialize, Clone)]
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

pub struct Diff {}

pub fn get_review_state(
    file_name: String,
    db: &DB,
    git: &Git,
) -> Result<StoredReviewForFile, Error> {
    let commit = db.latest_reviewed_commit(&file_name);
    let mut state = db.review_status_of_commit(&commit);
    let diff = git.diff_current_and_commit(commit, (state.exclusions).as_ref());
    if !diff.is_none() {
        state = transform_reviews(state, diff);
    }
    store_review_status(git.current_commit(), &state)?;
    Ok(review_status_of_file(file_name, state))
}

pub fn update_review_state(changes: UpdateReviewState, db: &DB, git: &Git) -> Result<(), Error> {
    let commit = db.latest_reviewed_commit(&changes.file_name);
    if (&commit).is_none() || commit.as_ref().unwrap() != &git.current_commit() {
        return Err(Error {
            reason: "Call GET first".to_string(),
        });
    }

    let state = db.review_status_of_commit(&commit);
    let new_state = update_reviews(state, changes);
    store_review_status(git.current_commit(), &new_state)?;
    return Ok(());
}

fn review_status_of_file(
    file_name: String,
    all_reviews: StoredReviewForCommit,
) -> StoredReviewForFile {
    match all_reviews.files.get(&file_name) {
        Some(state) => state.clone(),
        None => StoredReviewForFile {
            reviewed: HashSet::default(),
            modified: HashSet::default(),
        },
    }
}

fn transform_reviews(
    current_state: StoredReviewForCommit,
    diff: Option<Diff>,
) -> StoredReviewForCommit {
    current_state
}

fn update_reviews(
    current_state: StoredReviewForCommit,
    changes: UpdateReviewState,
) -> StoredReviewForCommit {
    current_state
}

fn store_review_status(commit: String, state: &StoredReviewForCommit) -> Result<(), Error> {
    Ok(())
}
