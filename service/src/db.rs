use serde::{Deserialize, Serialize};
use std::fs;
use std::hash::{Hash, Hasher};
use std::{
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    fs::File,
    io::{Read, Write},
    vec,
};
use uuid::Uuid;

use crate::{Comment, FileComments, MyError, StoredReviewForCommit, StoredReviewForFile};

#[derive(Serialize, Deserialize, Debug)]
struct DBForFile {
    file_name: String,
    latest_reviewed_commit: String,
    // Maps commit to reviews
    commit_reviews: HashMap<String, StoredReviewForFile>,
    comments: FileComments,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DB {
    db_dir: String,
    // Maps filepath to db data
    file_dbs: HashMap<String, DBForFile>,
    // Contains the list of excluded directories
    exclusions: Vec<String>,
}

impl DBForFile {
    pub fn default(file_name: String) -> Self {
        Self {
            file_name,
            latest_reviewed_commit: "".to_string(),
            commit_reviews: HashMap::default(),
            comments: FileComments(HashMap::default()),
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

        for path in paths {
            let path = path?;
            if path
                .file_name()
                .as_os_str()
                .to_str()
                .unwrap()
                .starts_with("db_")
            {
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

    pub fn save(&self) -> Result<(), MyError> {
        // let start = SystemTime::now();
        // let since_the_epoch = start
        //     .duration_since(UNIX_EPOCH)
        //     .expect("Time went backwards");
        // let backup_path = format!("{}-{}.db", self.path, since_the_epoch.as_millis());
        // if std::path::Path::new(&self.path).exists() {
        //     fs::rename(&self.path, backup_path)?;
        // }
        for (file_name, db_content) in &self.file_dbs {
            let ser = serde_json::to_string(&db_content)?;
            let mut s = DefaultHasher::new();
            file_name.hash(&mut s);
            let id_from_path = s.finish().to_string();
            let name: Vec<&str> = file_name.split("/").collect();
            let base_name = name.last().unwrap();
            let db_path = format!("{}/db_{}-{}.json", &self.db_dir, base_name, id_from_path);
            let mut output = File::create(db_path)?;
            output.write_all(ser.as_bytes())?;
        }
        Ok(())
    }

    pub fn latest_reviewed_commit(&self, file_name: &String) -> Option<String> {
        self.file_dbs
            .get(file_name)
            .and_then(|db| Some(db.latest_reviewed_commit.clone()))
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
                None => StoredReviewForFile {
                    modified: HashSet::default(),
                    reviewed: HashSet::default(),
                },
            };
            commit_reviews.files.insert(file_name.clone(), review);
        }
        commit_reviews
    }

    pub fn store_review_status(
        &mut self,
        commit: &String,
        state: &StoredReviewForCommit,
    ) -> Result<(), MyError> {
        for file_name in state.files.keys().into_iter() {
            let db_content = self
                .file_dbs
                .entry(file_name.clone())
                .or_insert(DBForFile::default(file_name.clone()));
            db_content.latest_reviewed_commit = commit.clone();
            db_content.commit_reviews.insert(
                commit.clone(),
                state
                    .files
                    .get(file_name)
                    .cloned()
                    .or_else(|| {
                        Some(StoredReviewForFile {
                            reviewed: HashSet::default(),
                            modified: HashSet::default(),
                        })
                    })
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
            .or_insert(DBForFile::default(file_name.clone()));
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

        let mut index = 0;
        for (i, comment) in current_comments.iter().enumerate() {
            if comment.id == comment_id {
                index = i;
                break;
            } else {
                return Err(MyError {
                    message: "comment id does not exist".to_string(),
                });
            }
        }

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

        let mut index = 0;
        for (i, comment) in current_comments.iter().enumerate() {
            if comment.id == comment_id {
                index = i;
                break;
            } else {
                return Err(MyError {
                    message: "comment id does not exist".to_string(),
                });
            }
        }

        let comment = current_comments.get_mut(index).unwrap();

        comment.body = body;
        comment.author = author;

        Ok(())
    }

    pub fn get_file_comments(&self, file_name: &String) -> Option<FileComments> {
        if let Some(db_content) = self.file_dbs.get(file_name) {
            Some(db_content.comments.clone())
        } else {
            None
        }
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
