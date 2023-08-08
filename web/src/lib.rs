use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ops::RangeInclusive, str::FromStr};
use bitflags::bitflags;

pub mod app;

#[derive(Serialize, Deserialize, Clone, Debug)]
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
    pub total_lines: usize,
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
        let percent = ((100 * total) as f32 / self.total_lines as f32) as usize;
        // TODO: fix this in the backend. The problem is that when review ranges are merged, they don't take into account that the file line count may have been reduced.
        if percent > 100 {
            100
        } else {
            percent
        }
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Priority {
    Unspecified,
    High,
    Medium,
    Low,
    Ignore,
}

bitflags! {
    // Specify the name of your flag set and the underlying integer type
    pub struct PriorityBF: u8 {
        const UNSPECIFIED = 0b1;
        const HIGH = 0b10;
        const MEDIUM = 0b100;
        const LOW = 0b1000;
        const IGNORE = 0b10000;
    }
}

impl FromStr for Priority {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Unspecified" => Ok(Priority::Unspecified),
            "High" => Ok(Priority::High),
            "Medium" => Ok(Priority::Medium),
            "Low" => Ok(Priority::Low),
            "Ignore" => Ok(Priority::Ignore),
            _ => Err(()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Metadata {
    priority: Priority,
    reviewer: String,
    note: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UpdateMetadataRequest {
    pub file_name: String,
    pub metadata: Metadata,
}

#[derive(Serialize, Deserialize, Clone)]
struct LatestFileInfo {
    file_name: String,
    line_reviews: StoredReviewForFile,
    comments: HashMap<usize, Vec<Comment>>,
    metadata: Option<Metadata>,
}

#[derive(Clone, Debug)]
pub struct Filters {
    pub only_with_comments: bool,
    pub only_c_files: bool,
    pub only_go_files: bool,

    pub sort_by_modified: bool,
    pub sort_by_reviewed: bool,
    pub sort_by_name: bool,

    pub reviewer_unassigned: bool,

    pub priority_mask: PriorityBF,
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

            reviewer_unassigned: false,

            priority_mask: PriorityBF::UNSPECIFIED | PriorityBF::HIGH | PriorityBF::MEDIUM | PriorityBF::LOW,
        }
    }
}
