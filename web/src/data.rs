use crate::{AllComments, Comment, CommentThread};
use std::{collections::HashMap, vec};

pub fn test_comment_data() -> AllComments {
    let file1 = "file1.cpp";
    let file2 = "path/file2.go";
    let mut file_comments = HashMap::default();
    file_comments.insert(
        file1.to_string(),
        vec![
            CommentThread::new(42, vec![Comment::new("Single comment", "Person3")]),
            CommentThread::new(
                11,
                vec![
                    Comment::new("Main comment", "Arash"),
                    Comment::new("Reply", "Person2"),
                ],
            ),
        ],
    );
    file_comments.insert(file2.to_string(), vec![]);

    AllComments { file_comments }
}
