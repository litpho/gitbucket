use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    // CLI error
    #[error("HOME environment variable not found")]
    HOMEEnvironmentVariableNotFound(#[source] std::env::VarError),

    // Git errors
    #[error("General git2 error")]
    FailedGitOperation(#[from] git2::Error),
    #[error("no branch found")]
    NoBranchFound,
    #[error("no branchname found")]
    NoBranchnameFound,

    // IO errors
    #[error("Reading directory {directory}")]
    FailedToReadDirectory {
        directory: String,
        source: std::io::Error,
    },

    // REST errors
    /// A general error from the Ureq library
    #[error("general ureq error")]
    FailedToDoHttpCall(#[from] Box<ureq::Error>),
    /// Invalid Username or Password
    #[error("invalid username or password")]
    InvalidUsernamePassword,
    /// A JSON parsing error
    #[error("JSON parsing error")]
    FailedToParseJSON(#[source] std::io::Error),
    /// SSH URL could not be found
    #[error("ssh url not found")]
    SshUrlMissing,
}
