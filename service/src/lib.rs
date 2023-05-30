use serde::{Deserialize, Serialize};

pub struct Error {
    pub reason: String,
}

#[derive(Serialize)]
pub struct ReviewState {
    reviewed: Vec<usize>,
    modified: Vec<usize>,
}

#[derive(Deserialize, Debug)]
pub enum State {
    Reviewed,
    Modified,
}

#[derive(Deserialize, Debug)]
pub struct UpdateReviewState {
    file_name: String,
    start_line: usize,
    end_line: usize,
    review_state: State,
}

pub fn get_review_state(file_name: String) -> ReviewState {
    ReviewState {
        reviewed: vec![0],
        modified: vec![2],
    }
}

pub fn update_review_state(changes: UpdateReviewState) -> Result<(), Error> {
    return Ok(());
}
