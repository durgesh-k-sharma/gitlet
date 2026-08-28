use std::fmt;

#[derive(Debug)]
pub enum GitletError {
    AlreadyExists,
    NotInitialized,
    FileNotFound,
    NoChangesAdded,
    EmptyCommitMessage,
    NoReasonToRemove,
    NoCommitWithId,
    NoFileInCommit,
    NoSuchBranch,
    BranchAlreadyExists,
    BranchDoesNotExist,
    CannotRemoveCurrentBranch,
    NoNeedToCheckoutCurrentBranch,
    UntrackedFileInWay,
    UncommittedChanges,
    CannotMergeWithSelf,
    FoundNoCommitWithMessage,
    IncorrectOperands,
    Io(std::io::Error),
    Other(String),
}

impl fmt::Display for GitletError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GitletError::AlreadyExists => {
                write!(
                    f,
                    "A Gitlet version-control system already exists in the current directory."
                )
            }
            GitletError::NotInitialized => write!(f, "Not in an initialized Gitlet directory."),
            GitletError::FileNotFound => write!(f, "File does not exist."),
            GitletError::NoChangesAdded => write!(f, "No changes added to the commit."),
            GitletError::EmptyCommitMessage => write!(f, "Please enter a commit message."),
            GitletError::NoReasonToRemove => write!(f, "No reason to remove the file."),
            GitletError::NoCommitWithId => write!(f, "No commit with that id exists."),
            GitletError::NoFileInCommit => write!(f, "File does not exist in that commit."),
            GitletError::NoSuchBranch => write!(f, "No such branch exists."),
            GitletError::BranchAlreadyExists => {
                write!(f, "A branch with that name already exists.")
            }
            GitletError::BranchDoesNotExist => write!(f, "A branch with that name does not exist."),
            GitletError::CannotRemoveCurrentBranch => {
                write!(f, "Cannot remove the current branch.")
            }
            GitletError::NoNeedToCheckoutCurrentBranch => {
                write!(f, "No need to checkout the current branch.")
            }
            GitletError::UntrackedFileInWay => {
                write!(
                    f,
                    "There is an untracked file in the way; delete it, or add and commit it first."
                )
            }
            GitletError::UncommittedChanges => write!(f, "You have uncommitted changes."),
            GitletError::CannotMergeWithSelf => write!(f, "Cannot merge a branch with itself."),
            GitletError::FoundNoCommitWithMessage => {
                write!(f, "Found no commit with that message.")
            }
            GitletError::IncorrectOperands => write!(f, "Incorrect operands."),
            GitletError::Io(err) => write!(f, "I/O error: {}", err),
            GitletError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for GitletError {}

impl From<std::io::Error> for GitletError {
    fn from(err: std::io::Error) -> Self {
        GitletError::Io(err)
    }
}

impl From<serde_json::Error> for GitletError {
    fn from(err: serde_json::Error) -> Self {
        GitletError::Other(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, GitletError>;
