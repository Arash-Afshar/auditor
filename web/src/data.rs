// use crate::{AllInfo, Comment, CommentThread, FileInfo, LineInfo};
// use std::{collections::HashMap, vec};
//
// pub fn test_comment_data() -> AllInfo {
//     let file1 = "file1.cpp";
//     let file2 = "path/file2.go";
//     let mut file_info = HashMap::default();
//     file_info.insert(
//         file1.to_string(),
//         FileInfo {
//             line_info: LineInfo {
//                 lines_reviewed: 10,
//                 lines_modified: 2,
//                 total_lines: 20,
//             },
//             comments: vec![
//                 CommentThread::new(42, vec![Comment::new("Single comment", "Person3")]),
//                 CommentThread::new(
//                     11,
//                     vec![
//                         Comment::new("Main comment", "Arash"),
//                         Comment::new("Reply", "Person2"),
//                     ],
//                 ),
//             ],
//         },
//     );
//     file_info.insert(
//         file2.to_string(),
//         FileInfo {
//             line_info: LineInfo {
//                 lines_reviewed: 30,
//                 lines_modified: 0,
//                 total_lines: 30,
//             },
//             comments: vec![],
//         },
//     );
//
//     AllInfo { file_info }
// }
//
