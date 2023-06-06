use std::collections::HashMap;

pub mod app;
pub mod data;

#[derive(Clone)]
pub struct Comment {
    content: String,
    author: String,
}

#[derive(Clone)]
pub struct CommentThread {
    pub line_number: u32,
    pub content: Vec<Comment>,
}

#[derive(Clone)]
pub struct LineInfo {
    pub lines_reviewed: usize,
    pub lines_modified: usize,
    pub total_lines: usize,
}

#[derive(Clone)]
pub struct FileInfo {
    pub line_info: LineInfo,
    pub comments: Vec<CommentThread>,
}

#[derive(Clone)]
pub struct AllInfo {
    // file_name -> FileInfo
    pub file_info: HashMap<String, FileInfo>,
}

impl Comment {
    fn new(content: &str, author: &str) -> Self {
        Self {
            content: content.to_string(),
            author: author.to_string(),
        }
    }
}

impl CommentThread {
    fn new(line_number: u32, content: Vec<Comment>) -> Self {
        Self {
            line_number,
            content,
        }
    }
}
