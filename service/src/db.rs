use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
    time::{SystemTime, UNIX_EPOCH},
    vec,
};

use crate::{MyError, StoredReviewForCommit};

#[derive(Serialize, Deserialize, Debug)]
pub struct DB {
    path: String,
    // Maps filepath to commit
    latest_reviewed_commit_for_file: HashMap<String, String>,
    // Maps commit to reviews
    commit_reviews: HashMap<String, StoredReviewForCommit>,
    // Contains the list of excluded directories
    exclusions: Vec<String>,
}

impl DB {
    pub fn new(path: String) -> Result<Self, MyError> {
        match File::open(&path) {
            Ok(mut input) => {
                let mut contents = String::new();
                input.read_to_string(&mut contents)?;
                let deserialized: DB = serde_json::from_str(&contents)?;

                Ok(deserialized)
            }
            Err(_) => Ok(Self {
                path,
                latest_reviewed_commit_for_file: HashMap::default(),
                commit_reviews: HashMap::default(),
                exclusions: vec![],
            }),
        }
    }

    pub fn save(&self) -> Result<(), MyError> {
        // let start = SystemTime::now();
        // let since_the_epoch = start
        //     .duration_since(UNIX_EPOCH)
        //     .expect("Time went backwards");
        // let backup_path = format!("{}-{}.db", self.path, since_the_epoch.as_millis());
        // if std::path::Path::new(&self.path).exists() {
        //     fs::rename(&self.path, backup_path)?;
        // }
        let ser = serde_json::to_string(&self)?;
        let mut output = File::create(&self.path)?;
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

    pub fn store_review_status(
        &mut self,
        commit: &String,
        state: &StoredReviewForCommit,
    ) -> Result<(), MyError> {
        for file in state.files.keys().into_iter() {
            self.latest_reviewed_commit_for_file
                .insert(file.clone(), commit.clone());
        }
        self.commit_reviews.insert(commit.clone(), state.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::*;

    #[test]
    fn test_read_write_scenarios() {
        let path = "test.db".to_string();
        let commit = "commit1".to_string();
        let file1 = "file1".to_string();
        let file2 = "file2".to_string();
        let mut file_reviews = HashMap::default();
        file_reviews.insert(
            file1.clone(),
            StoredReviewForFile {
                reviewed: HashSet::from_iter(vec![0]),
                modified: HashSet::from_iter(vec![1]),
            },
        );
        let state = &StoredReviewForCommit {
            files: file_reviews,
            exclusions: vec![],
        };

        let mut db = DB::new(path.clone()).unwrap();
        db.store_review_status(&commit, state).unwrap();
        db.save().unwrap();
        let db = DB::new(path.clone()).unwrap();

        assert_eq!(db.latest_reviewed_commit(&file1), Some(commit.clone()));
        assert_eq!(db.latest_reviewed_commit(&file2), None);
        let retrieved_state = db.review_status_of_commit(&Some(commit));
        assert_eq!(state.exclusions, retrieved_state.exclusions);
        assert!(retrieved_state.files.get(&file1).is_some());
        assert!(retrieved_state.files.get(&file2).is_none());
        assert_eq!(
            state.files.get(&file1).unwrap(),
            retrieved_state.files.get(&file1).unwrap()
        );
    }

    #[test]
    fn test_inspect_db() {
        // TODO: make this into a cmd
        let path = "main.db".to_string();
        let db = DB::new(path.clone()).unwrap();
        println!("{:?}", db);
    }
}
