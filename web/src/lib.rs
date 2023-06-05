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
pub struct AllComments {
    // file_name -> Comment thread
    pub file_comments: HashMap<String, Vec<CommentThread>>,
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
