use thiserror::Error;

/// WordCountError enumerates all possible errors returned by this library.
#[derive(Error, Debug)]
pub enum AuditorError {
    #[error("Config is missing an item")]
    MissingConfig(String),

    #[error("Comment id is not found")]
    UnknownCommentId(String),

    #[error("Commit is older than the latest commit")]
    OldCommitError(String),

    #[error("Commit in db is newer than the latest commit")]
    ShouldUpdateToLatest(String),

    #[error("Commit not found in the db")]
    UnknownCommit(String),

    #[error("Filename not found in the db")]
    UnknownFileName(String),

    #[error("The path name should have at least one / in it")]
    InvalidAbsolutePath(String),

    #[error("The line number does not exits")]
    UnknownLinenumberInFile(usize, String),

    #[error("OsString is None")]
    OsStringError,

    #[error(transparent)]
    RegexError(#[from] regex::Error),

    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),

    #[error(transparent)]
    GitError(#[from] git2::Error),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    ConfigError(#[from] toml::de::Error),
}
