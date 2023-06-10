use std::{collections::HashMap, ops::RangeInclusive};

use serde::{Deserialize, Serialize};

pub mod app;
pub mod data;

#[derive(Serialize, Deserialize, Clone)]
pub struct Comment {
    pub id: String,
    pub body: String,
    pub author: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct StoredReviewForFile {
    pub reviewed: Vec<RangeInclusive<usize>>,
    pub modified: Vec<RangeInclusive<usize>>,
    pub ignored: Vec<RangeInclusive<usize>>,
    total_lines: usize,
}

impl StoredReviewForFile {
    fn percent_helper(&self, list: &Vec<RangeInclusive<usize>>) -> usize {
        if self.total_lines == 0 {
            return 0;
        }
        let mut total = 0;
        for range in list {
            total += *range.end() - range.start() + 1;
        }
        ((100 * total) as f32 / self.total_lines as f32) as usize
    }

    fn percent_reviewed(&self) -> usize {
        self.percent_helper(&self.reviewed)
    }

    fn percent_modified(&self) -> usize {
        self.percent_helper(&self.modified)
    }

    fn percent_ignored(&self) -> usize {
        self.percent_helper(&self.ignored)
    }
}
#[derive(Serialize, Deserialize, Clone)]
struct LatestFileInfos(Vec<LatestFileInfo>);

#[derive(Serialize, Deserialize, Clone)]
pub struct LatestFileInfo {
    file_name: String,
    line_reviews: StoredReviewForFile,
    comments: HashMap<usize, Vec<Comment>>,
}

#[derive(Clone, Debug)]
pub struct Filters {
    pub only_with_comments: bool,
    pub only_c_files: bool,
    pub only_go_files: bool,
    pub sort_by_modified: bool,
    pub sort_by_reviewed: bool,
    pub sort_by_name: bool,
}

impl Default for Filters {
    fn default() -> Self {
        Self {
            only_with_comments: false,
            only_c_files: true,
            only_go_files: false,
            sort_by_modified: false,
            sort_by_reviewed: true,
            sort_by_name: false,
        }
    }
}
