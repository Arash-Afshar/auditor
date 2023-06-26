use anyhow::Result;
use db::DB;
use errors::AuditorError;
use git::Git;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ops::RangeInclusive};
pub mod config;
pub mod db;
pub mod errors;
pub mod git;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Comment {
    pub id: String,
    pub body: String,
    pub author: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileComments(pub HashMap<usize, Vec<Comment>>);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Priority {
    High,
    Medium,
    Low,
    Ignore,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Metadata {
    priority: Priority,
}

#[derive(Deserialize, Debug)]
pub struct UpdateMetadataRequest {
    pub file_name: String,
    pub metadata: Metadata,
}

#[derive(Deserialize, Debug)]
pub enum State {
    Reviewed,
    Modified,
    Ignored,
    Cleared,
}

#[derive(Deserialize, Debug)]
pub struct UpdateReviewState {
    pub file_name: String,
    start_line: usize,
    end_line: usize,
    review_state: State,
    total_lines: usize,
}

impl UpdateReviewState {
    fn range(&self) -> RangeInclusive<usize> {
        RangeInclusive::new(self.start_line, self.end_line)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct StoredReviewForFile {
    pub reviewed: Vec<RangeInclusive<usize>>,
    pub modified: Vec<RangeInclusive<usize>>,
    pub ignored: Vec<RangeInclusive<usize>>,
    pub total_lines: usize,
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

impl StoredReviewForFile {
    fn default() -> Self {
        Self {
            reviewed: vec![],
            modified: vec![],
            ignored: vec![],
            total_lines: 0,
        }
    }

    fn new(state: &State, range: RangeInclusive<usize>, total_lines: usize) -> Self {
        let mut instance = Self::default();
        instance.total_lines = total_lines;
        match state {
            State::Reviewed => instance.reviewed.push(range),
            State::Modified => instance.modified.push(range),
            State::Ignored => instance.ignored.push(range),
            State::Cleared => (),
        };
        instance
    }

    fn mark_lines(&mut self, state: &State, new_range: &RangeInclusive<usize>) {
        // Add the changes to the current field
        match state {
            State::Reviewed => {
                self.reviewed = Self::add_range_to_list(new_range.clone(), self.reviewed.clone())
            }
            State::Modified => {
                self.modified = Self::add_range_to_list(new_range.clone(), self.modified.clone())
            }
            State::Ignored => {
                self.ignored = Self::add_range_to_list(new_range.clone(), self.ignored.clone())
            }
            State::Cleared => (),
        };

        // Remove range from all other fields
        match state {
            State::Reviewed => {
                self.modified = Self::remove_overlapping_range(new_range, &self.modified);
                self.ignored = Self::remove_overlapping_range(new_range, &self.ignored);
            }
            State::Modified => {
                self.reviewed = Self::remove_overlapping_range(new_range, &self.reviewed);
                self.ignored = Self::remove_overlapping_range(new_range, &self.ignored);
            }
            State::Ignored => {
                self.reviewed = Self::remove_overlapping_range(new_range, &self.reviewed);
                self.modified = Self::remove_overlapping_range(new_range, &self.modified);
            }
            State::Cleared => {
                self.reviewed = Self::remove_overlapping_range(new_range, &self.reviewed);
                self.modified = Self::remove_overlapping_range(new_range, &self.modified);
                self.ignored = Self::remove_overlapping_range(new_range, &self.ignored);
            }
        };
    }

    fn add_range_to_list(
        new_range: RangeInclusive<usize>,
        target_ranges: Vec<RangeInclusive<usize>>,
    ) -> Vec<RangeInclusive<usize>> {
        let mut new_start = *new_range.start();
        let mut new_end = *new_range.end();
        let mut updated_ranges: Vec<RangeInclusive<usize>> = vec![];
        let mut is_added = false;
        for (i, range) in target_ranges.clone().into_iter().enumerate() {
            if new_end + 1 < *range.start() {
                updated_ranges.push(RangeInclusive::new(new_start, new_end));
                let tail = target_ranges[i..].to_vec();
                updated_ranges.extend(tail);
                is_added = true;
                break;
            } else if new_start > *range.end() + 1 {
                updated_ranges.push(range);
            } else {
                new_start = std::cmp::min(*range.start(), new_start);
                new_end = std::cmp::max(*range.end(), new_end);
            }
        }
        if !is_added {
            updated_ranges.push(RangeInclusive::new(new_start, new_end));
        }
        updated_ranges
    }

    fn remove_overlapping_range(
        new_range: &RangeInclusive<usize>,
        ranges: &[RangeInclusive<usize>],
    ) -> Vec<RangeInclusive<usize>> {
        let mut updated_ranges: Vec<RangeInclusive<usize>> = vec![];
        for (i, range) in ranges.iter().enumerate() {
            if new_range.end() < range.start() {
                let tail = ranges[i..].to_vec();
                updated_ranges.extend(tail);
                break;
            } else if new_range.start() > range.end() {
                updated_ranges.push(range.clone());
            } else {
                if range.start() < new_range.start() {
                    updated_ranges.push(RangeInclusive::new(*range.start(), new_range.start() - 1));
                }
                if new_range.end() < range.end() {
                    updated_ranges.push(RangeInclusive::new(new_range.end() + 1, *range.end()));
                }
            }
        }
        updated_ranges
    }
}

pub fn get_review_state(file_name: &String, db: &DB) -> Result<StoredReviewForFile> {
    let commit = db.latest_reviewed_commit(file_name);
    let state = db.review_status_of_commit(&commit);
    Ok(match state.files.get(file_name) {
        Some(state) => state.clone(),
        None => StoredReviewForFile::default(),
    })
}

pub fn transform_review_state(
    file_name: &String,
    db: &mut DB,
    git: &Git,
) -> Result<StoredReviewForFile> {
    let commit = db.latest_reviewed_commit(file_name);
    let mut state = db.review_status_of_commit(&commit);
    let diff = git.diff_current_and_commit(commit, (state.exclusions).as_ref())?;
    if diff.is_some() {
        state = transform_reviews(&state, diff);
        db.store_review_status(&git.current_commit()?, &state)?;
    }
    Ok(match state.files.get(file_name) {
        Some(state) => state.clone(),
        None => StoredReviewForFile::default(),
    })
}

pub fn update_review_state(changes: UpdateReviewState, db: &mut DB, git: &Git) -> Result<()> {
    let commit = db.latest_reviewed_commit(&changes.file_name);
    let state = db.review_status_of_commit(&commit);
    let new_state = update_reviews(&state, changes);
    db.store_review_status(&git.current_commit()?, &new_state)?;
    Ok(())
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
        let file_review = new_state
            .files
            .entry(file_name)
            .or_insert(StoredReviewForFile::default());
        for line_diff in line_diffs {
            if let Some(new_line) = line_diff.new {
                let new_line_number: usize = new_line.try_into().unwrap();
                let new_line_number = new_line_number - 1;
                file_review.mark_lines(
                    &State::Modified,
                    &RangeInclusive::new(new_line_number, new_line_number),
                );
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
    let mut new_state = current_state.clone();
    let file_reviews = new_state.files.get_mut(&changes.file_name);
    match file_reviews {
        Some(current_file_reviews) => {
            current_file_reviews.mark_lines(&changes.review_state, &changes.range());
            current_file_reviews.total_lines = changes.total_lines;
        }
        None => {
            let new_file_review = StoredReviewForFile::new(
                &changes.review_state,
                changes.range(),
                changes.total_lines,
            );
            new_state
                .files
                .insert(changes.file_name.clone(), new_file_review);
        }
    };
    new_state
}

pub fn update_metadata(request: UpdateMetadataRequest, db: &mut DB) -> Result<()> {
    db.set_metadata(&request.file_name, request.metadata)
}

#[cfg(test)]
mod tests {

    use super::*;

    fn range(r: (usize, usize)) -> RangeInclusive<usize> {
        RangeInclusive::new(r.0, r.1)
    }

    fn ranges(rs: Vec<(usize, usize)>) -> Vec<RangeInclusive<usize>> {
        rs.into_iter().map(range).collect()
    }

    #[test]
    fn test_transform_reviews() {
        let file1 = "file1".to_string();
        let file2 = "file2".to_string();
        let mut files: HashMap<String, StoredReviewForFile> = HashMap::default();
        files.insert(
            file1.clone(),
            StoredReviewForFile {
                reviewed: ranges(vec![(0, 0)]),
                modified: ranges(vec![(1, 1)]),
                ignored: ranges(vec![(2, 2)]),
                total_lines: 0, // TODO: add tests for this case
            },
        );
        let current_state = &StoredReviewForCommit {
            files,
            exclusions: vec![],
        };

        // ----------- test: Empty diff has no effect
        let diff = None;
        let state = transform_reviews(current_state, diff);
        assert_eq!(current_state.files, state.files);

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
        let state = &transform_reviews(&state, diff);
        let access = |file, state: &StoredReviewForCommit| state.files.get(file).unwrap().clone();
        assert_eq!(access(&file1, state).reviewed, ranges(vec![(0, 0)]));
        assert_eq!(access(&file1, state).modified, ranges(vec![(1, 1)]));
        assert_eq!(access(&file2, state).reviewed, ranges(vec![]));
        assert_eq!(access(&file2, state).modified, ranges(vec![(2, 2)]));

        // ----------- test: Diff causes modified lines - existing
        let mut diff_files = HashMap::default();
        diff_files.insert(
            file1.clone(),
            vec![LineDiff {
                old: None,
                new: Some(1),
            }],
        );
        let diff = Some(Diff { files: diff_files });
        let state = &transform_reviews(state, diff);
        assert_eq!(access(&file1, state).reviewed, ranges(vec![]));
        assert_eq!(access(&file1, state).modified, ranges(vec![(0, 1)]));
        assert_eq!(access(&file2, state).reviewed, ranges(vec![]));
        assert_eq!(access(&file2, state).modified, ranges(vec![(2, 2)]));
    }

    #[test]
    fn test_update_reviews() {
        let file1 = "file1".to_string();
        let mut files: HashMap<String, StoredReviewForFile> = HashMap::default();
        files.insert(
            file1.clone(),
            StoredReviewForFile {
                reviewed: vec![RangeInclusive::new(0, 0)],
                modified: vec![RangeInclusive::new(1, 1)],
                ignored: vec![RangeInclusive::new(2, 2)],
                total_lines: 0, // TODO: add tests for this case
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
            total_lines: 0,
        };
        let state = &update_reviews(current_state, changes);
        let access = |file, state: &StoredReviewForCommit| state.files.get(file).unwrap().clone();
        assert_eq!(access(&file1, state).reviewed, ranges(vec![(0, 0), (3, 5)]));
        assert_eq!(access(&file1, state).modified, ranges(vec![(1, 1)]));
        assert_eq!(access(&file1, state).ignored, ranges(vec![(2, 2)]));

        // --------------- test change to modified
        let changes = UpdateReviewState {
            file_name: file1.clone(),
            start_line: 2,
            end_line: 4,
            review_state: State::Modified,
            total_lines: 0,
        };
        let state = &update_reviews(state, changes);
        assert_eq!(access(&file1, state).reviewed, ranges(vec![(0, 0), (5, 5)]));
        assert_eq!(access(&file1, state).modified, ranges(vec![(1, 4)]));
        assert_eq!(access(&file1, state).ignored, ranges(vec![]));

        // --------------- test change to reviewed
        let changes = UpdateReviewState {
            file_name: file1.clone(),
            start_line: 2,
            end_line: 3,
            review_state: State::Reviewed,
            total_lines: 0,
        };
        let state = &update_reviews(state, changes);
        assert_eq!(
            access(&file1, state).reviewed,
            ranges(vec![(0, 0), (2, 3), (5, 5)])
        );
        assert_eq!(access(&file1, state).modified, ranges(vec![(1, 1), (4, 4)]));
        assert_eq!(access(&file1, state).ignored, ranges(vec![]));

        // --------------- test clear
        let changes = UpdateReviewState {
            file_name: file1.clone(),
            start_line: 1,
            end_line: 5,
            review_state: State::Cleared,
            total_lines: 0,
        };
        let state = &update_reviews(state, changes);
        assert_eq!(access(&file1, state).reviewed, ranges(vec![(0, 0)]));
        assert_eq!(access(&file1, state).modified, ranges(vec![]));
        assert_eq!(access(&file1, state).ignored, ranges(vec![]));
        // assert_eq!(
        //     updated_state.files.get(&file1).unwrap().clone().reviewed,
        //     vec![RangeInclusive::new(0, 0)]
        // );
        // assert_eq!(
        //     updated_state.files.get(&file1).unwrap().clone().modified,
        //     vec![]
        // );
    }

    #[test]
    fn test_mark_lines() {
        // Add to empty ranges
        let res = StoredReviewForFile::add_range_to_list(range((4, 6)), ranges(vec![]));
        assert_eq!(ranges(vec![(4, 6)]), res);

        // Add no overlap in the begining
        let res = StoredReviewForFile::add_range_to_list(range((4, 6)), ranges(vec![(8, 9)]));
        assert_eq!(ranges(vec![(4, 6), (8, 9)]), res);
        let res =
            StoredReviewForFile::add_range_to_list(range((4, 6)), ranges(vec![(8, 9), (11, 12)]));
        assert_eq!(ranges(vec![(4, 6), (8, 9), (11, 12)]), res);

        // Add no overlap in the end
        let res = StoredReviewForFile::add_range_to_list(range((4, 6)), ranges(vec![(1, 2)]));
        assert_eq!(ranges(vec![(1, 2), (4, 6)]), res);
        let res =
            StoredReviewForFile::add_range_to_list(range((9, 13)), ranges(vec![(1, 2), (4, 5)]));
        assert_eq!(ranges(vec![(1, 2), (4, 5), (9, 13)]), res);

        // Add no overlap in the middle
        let res =
            StoredReviewForFile::add_range_to_list(range((4, 6)), ranges(vec![(1, 2), (8, 9)]));
        assert_eq!(ranges(vec![(1, 2), (4, 6), (8, 9)]), res);

        // Add no overlap but merge them
        let res = StoredReviewForFile::add_range_to_list(range((1, 2)), ranges(vec![(3, 4)]));
        assert_eq!(ranges(vec![(1, 4)]), res);
        let res = StoredReviewForFile::add_range_to_list(range((5, 6)), ranges(vec![(3, 4)]));
        assert_eq!(ranges(vec![(3, 6)]), res);

        // Add with overlap in the beginning
        let res =
            StoredReviewForFile::add_range_to_list(range((4, 9)), ranges(vec![(8, 10), (20, 22)]));
        assert_eq!(ranges(vec![(4, 10), (20, 22)]), res);

        // Add with overlap in the end
        let res = StoredReviewForFile::add_range_to_list(
            range((21, 30)),
            ranges(vec![(8, 10), (20, 22)]),
        );
        assert_eq!(ranges(vec![(8, 10), (20, 30)]), res);

        // Add with overlap in the middle
        let res =
            StoredReviewForFile::add_range_to_list(range((9, 21)), ranges(vec![(8, 10), (20, 22)]));
        assert_eq!(ranges(vec![(8, 22)]), res);

        // Add with overlap boundary start
        let res = StoredReviewForFile::add_range_to_list(range((9, 21)), ranges(vec![(7, 9)]));
        assert_eq!(ranges(vec![(7, 21)]), res);

        // Add with overlap boundary end
        let res = StoredReviewForFile::add_range_to_list(range((9, 21)), ranges(vec![(21, 22)]));
        assert_eq!(ranges(vec![(9, 22)]), res);
    }

    #[test]
    fn test_remove_overlapping_range() {
        // No overlap
        let res = StoredReviewForFile::remove_overlapping_range(
            &range((1, 2)),
            &ranges(vec![(3, 4), (7, 9)]),
        );
        assert_eq!(ranges(vec![(3, 4), (7, 9)]), res);

        // Overlap in the begining
        let res = StoredReviewForFile::remove_overlapping_range(
            &range((1, 4)),
            &ranges(vec![(3, 5), (7, 9)]),
        );
        assert_eq!(ranges(vec![(5, 5), (7, 9)]), res);
        let res = StoredReviewForFile::remove_overlapping_range(
            &range((1, 3)),
            &ranges(vec![(3, 5), (7, 9)]),
        );
        assert_eq!(ranges(vec![(4, 5), (7, 9)]), res);

        // Overlap in the end
        let res = StoredReviewForFile::remove_overlapping_range(
            &range((8, 10)),
            &ranges(vec![(3, 5), (7, 9)]),
        );
        assert_eq!(ranges(vec![(3, 5), (7, 7)]), res);
        let res = StoredReviewForFile::remove_overlapping_range(
            &range((9, 10)),
            &ranges(vec![(3, 5), (7, 9)]),
        );
        assert_eq!(ranges(vec![(3, 5), (7, 8)]), res);

        // Overlap in the middle
        let res = StoredReviewForFile::remove_overlapping_range(
            &range((4, 7)),
            &ranges(vec![(3, 5), (7, 9)]),
        );
        assert_eq!(ranges(vec![(3, 3), (8, 9)]), res);
        let res = StoredReviewForFile::remove_overlapping_range(
            &range((5, 9)),
            &ranges(vec![(3, 5), (7, 9)]),
        );
        assert_eq!(ranges(vec![(3, 4)]), res);
    }
}
