use crate::Diff;

pub struct Git {}

impl Git {
    pub fn new() -> Self {
        Git {}
    }

    pub fn current_commit(&self) -> String {
        "".to_string()
    }

    pub fn diff_current_and_commit(
        &self,
        commit: Option<String>,
        exclusions: &Vec<String>,
    ) -> Option<Diff> {
        None
    }
}
