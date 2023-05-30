use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Display,
    fs::File,
    io::{Read, Write},
    vec,
};

use crate::StoredReviewForCommit;

// TODO: rewrite with thiserror
pub struct MyError {
    pub code: i32,
    pub message: String,
}

impl<E: Display> From<E> for MyError {
    fn from(value: E) -> Self {
        MyError {
            code: 0,
            message: value.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct DB {
    // Maps filepath to commit
    latest_reviewed_commit_for_file: HashMap<String, String>,
    commit_reviews: HashMap<String, StoredReviewForCommit>,
    exclusions: Vec<String>,
}

impl DB {
    pub fn new() -> Self {
        DB {
            latest_reviewed_commit_for_file: HashMap::default(),
            commit_reviews: HashMap::default(),
            exclusions: vec![],
        }
    }

    pub fn load(&mut self, src: &str) -> Result<(), MyError> {
        let mut input = File::open(src)?;
        let mut contents = String::new();
        input.read_to_string(&mut contents)?;
        let deserialized: DB = serde_json::from_str(&contents)?;
        self.latest_reviewed_commit_for_file = deserialized.latest_reviewed_commit_for_file;
        Ok(())
    }

    pub fn save(self, dst: &str) -> Result<(), MyError> {
        let ser = serde_json::to_string(&self)?;
        let mut output = File::create(dst)?;
        output.write_all(ser.as_bytes())?;
        Ok(())
    }

    pub fn latest_reviewed_commit(&self, file_name: &String) -> Option<String> {
        self.latest_reviewed_commit_for_file.get(file_name).cloned()
    }

    pub fn review_status_of_commit(&self, commit: &Option<String>) -> StoredReviewForCommit {
        match commit {
            Some(commit) => match self.commit_reviews.get(commit) {
                Some(review) => review.clone(),
                None => StoredReviewForCommit::new(self.exclusions.clone()),
            },
            None => StoredReviewForCommit::new(self.exclusions.clone()),
        }
    }
}
