use crate::{
    Comment, FileComments, Metadata, MyError, Priority, StoredReviewForCommit, StoredReviewForFile,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::hash::{Hash, Hasher};
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fs::File,
    io::{Read, Write},
    vec,
};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct DBForFile {
    file_name: String,
    total_lines: usize,
    latest_reviewed_commit: String,
    // Maps commit to reviews
    commit_reviews: HashMap<String, StoredReviewForFile>,
    comments: FileComments,
    metadata: Option<Metadata>,
}

impl DBForFile {
    pub fn get_latest_info(&self) -> (String, StoredReviewForFile, FileComments, Option<Priority>) {
        (
            self.file_name.clone(),
            self.commit_reviews
                .get(&self.latest_reviewed_commit)
                .unwrap()
                .clone(),
            self.comments.clone(),
            self.metadata.as_ref().map(|m| m.priority.clone()),
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DB {
    db_dir: String,
    // Maps filepath to db data
    pub file_dbs: HashMap<String, DBForFile>,
    // Contains the list of excluded directories
    exclusions: Vec<String>,
}

impl DBForFile {
    pub fn default(file_name: String) -> Self {
        Self {
            file_name,
            total_lines: 0,
            latest_reviewed_commit: "".to_string(),
            commit_reviews: HashMap::default(),
            comments: FileComments(HashMap::default()),
            metadata: None,
        }
    }
}

impl DB {
    pub fn new(db_dir: String) -> Result<Self, MyError> {
        let paths = fs::read_dir(db_dir.clone()).unwrap();
        let mut db = Self {
            db_dir,
            exclusions: vec![],
            file_dbs: HashMap::default(),
        };

        let re = Regex::new(r"^db_.*\.(go|cpp|c|h)-\d*\.json$")?;
        for path in paths {
            let path = path?;
            let base_name = path.file_name();
            let base_name = base_name.as_os_str().to_str().unwrap();

            if re.is_match(base_name) {
                //if base_name.starts_with("db_") {
                let path = path.path();
                let path = path.to_str().unwrap();
                let mut file = File::open(path)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                let deserialized: DBForFile = serde_json::from_str(&contents)?;
                db.file_dbs
                    .insert(deserialized.file_name.clone(), deserialized);
            }
        }
        Ok(db)
    }

    pub fn new_single_file(db_dir: String, file_name: &String) -> Result<Self, MyError> {
        let mut db = Self {
            db_dir: db_dir.clone(),
            exclusions: vec![],
            file_dbs: HashMap::default(),
        };

        let path = Self::stored_file_name(file_name);
        let path = format!("{db_dir}/{path}");
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let deserialized: DBForFile = serde_json::from_str(&contents)?;
        db.file_dbs
            .insert(deserialized.file_name.clone(), deserialized);
        Ok(db)
    }

    pub fn save(&self) -> Result<(), MyError> {
        self.file_dbs.iter().for_each(|(file_name, _)| {
            self.save_file(file_name).unwrap();
        });
        Ok(())
    }

    pub fn save_file(&self, file_name: &String) -> Result<(), MyError> {
        let db_content = self.file_dbs.get(file_name).unwrap();
        let ser = serde_json::to_string(&db_content)?;
        let db_path = format!("{}/{}", &self.db_dir, Self::stored_file_name(file_name));
        let mut output = File::create(db_path)?;
        output.write_all(ser.as_bytes())?;
        Ok(())
    }

    fn stored_file_name(file_name: &String) -> String {
        let mut s = DefaultHasher::new();
        file_name.hash(&mut s);
        let id_from_path = s.finish().to_string();
        let name: Vec<&str> = file_name.split('/').collect();
        let base_name = name.last().unwrap();
        format!("db_{}-{}.json", base_name, id_from_path)
    }

    pub fn latest_reviewed_commit(&self, file_name: &String) -> Option<String> {
        self.file_dbs
            .get(file_name)
            .map(|db| db.latest_reviewed_commit.clone())
    }

    pub fn review_status_of_commit(&self, commit: &Option<String>) -> StoredReviewForCommit {
        let mut commit_reviews = StoredReviewForCommit {
            exclusions: vec![],
            files: HashMap::default(),
        };
        if commit.is_none() {
            return StoredReviewForCommit::new(self.exclusions.clone());
        }
        let commit = commit.clone().unwrap();
        for (file_name, db_content) in &self.file_dbs {
            let review = match db_content.commit_reviews.get(&commit) {
                Some(review) => review.clone(),
                None => StoredReviewForFile::default(),
            };
            commit_reviews.files.insert(file_name.clone(), review);
        }
        commit_reviews
    }

    pub fn store_review_status(
        &mut self,
        commit: &str,
        state: &StoredReviewForCommit,
    ) -> Result<(), MyError> {
        for file_name in state.files.keys() {
            let db_content = self
                .file_dbs
                .entry(file_name.clone())
                .or_insert(DBForFile::default(file_name.clone()));
            db_content.latest_reviewed_commit = commit.to_string();
            db_content.commit_reviews.insert(
                commit.to_string(),
                state
                    .files
                    .get(file_name)
                    .cloned()
                    .or_else(|| Some(StoredReviewForFile::default()))
                    .unwrap()
                    .clone(),
            );
        }
        Ok(())
    }

    pub fn add_new_comment(
        &mut self,
        file_name: String,
        line_number: usize,
        body: String,
        author: String,
    ) -> Result<String, MyError> {
        let id = Uuid::new_v4().to_string();
        let comment = Comment {
            id: id.clone(),
            body,
            author,
        };
        let db_content = self
            .file_dbs
            .entry(file_name.clone())
            .or_insert(DBForFile::default(file_name));
        let current_comments = db_content.comments.0.entry(line_number).or_insert(vec![]);
        current_comments.push(comment);
        Ok(id)
    }

    pub fn delete_comment(
        &mut self,
        file_name: String,
        comment_id: String,
        line_number: usize,
    ) -> Result<(), MyError> {
        let current_comments = self
            .file_dbs
            .get_mut(&file_name)
            .unwrap()
            .comments
            .0
            .get_mut(&line_number)
            .unwrap();

        let mut index = None;
        for (i, comment) in current_comments.iter().enumerate() {
            if comment.id == comment_id {
                index = Some(i);
                break;
            }
        }
        if index.is_none() {
            return Err(MyError {
                message: "comment id does not exist".to_string(),
            });
        }
        let index = index.unwrap();

        current_comments.remove(index);
        if current_comments.is_empty() {
            self.file_dbs
                .get_mut(&file_name)
                .unwrap()
                .comments
                .0
                .remove(&line_number);
        }
        Ok(())
    }

    pub fn update_comment(
        &mut self,
        file_name: String,
        comment_id: String,
        line_number: usize,
        body: String,
        author: String,
    ) -> Result<(), MyError> {
        let current_comments = self
            .file_dbs
            .get_mut(&file_name)
            .unwrap()
            .comments
            .0
            .get_mut(&line_number)
            .unwrap();

        let mut index = None;
        for (i, comment) in current_comments.iter().enumerate() {
            if comment.id == comment_id {
                index = Some(i);
                break;
            }
        }
        if index.is_none() {
            return Err(MyError {
                message: "comment id does not exist".to_string(),
            });
        }
        let index = index.unwrap();

        let comment = current_comments.get_mut(index).unwrap();

        comment.body = body;
        comment.author = author;

        Ok(())
    }

    pub fn get_file_comments(&self, file_name: &String) -> Option<FileComments> {
        self.file_dbs
            .get(file_name)
            .map(|db_content| db_content.comments.clone())
    }

    pub fn set_metadata(&mut self, file_name: &String, metadata: Metadata) {
        let db_content = self.file_dbs.get_mut(file_name).unwrap();
        db_content.metadata = Some(metadata);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::*;

    #[test]
    fn test_read_write_scenarios() {
        let path = ".".to_string();
        let commit = "commit1".to_string();
        let file1 = "file1".to_string();
        let file2 = "file2".to_string();
        let mut file_reviews = HashMap::default();
        file_reviews.insert(
            file1.clone(),
            StoredReviewForFile {
                reviewed: vec![RangeInclusive::new(0, 0)],
                modified: vec![RangeInclusive::new(1, 1)],
                ignored: vec![], // TODO: add tests for this case
                total_lines: 0,  // TODO: add tests for this case
            },
        );
        let state = &StoredReviewForCommit {
            files: file_reviews,
            exclusions: vec![],
        };

        let mut db = DB::new(path.clone()).unwrap();
        db.store_review_status(&commit, state).unwrap();
        db.save().unwrap();
        let db = DB::new(path).unwrap();

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
        // // TODO: make this into a cmd
        // let path = "main.db".to_string();
        // let db = DB::new(path).unwrap();
        // println!("{:?}", db);
    }
}
