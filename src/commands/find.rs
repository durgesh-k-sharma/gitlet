use crate::error::{GitletError, Result};
use crate::repo::Repo;

pub fn execute(target_message: &str) -> Result<()> {
    let repo = Repo::find()?;
    let all_commits = repo.get_all_commits()?;

    let mut matches = Vec::new();
    for (id, commit) in all_commits {
        if commit.message == target_message {
            matches.push(id);
        }
    }

    if matches.is_empty() {
        return Err(GitletError::FoundNoCommitWithMessage);
    }

    for id in matches {
        println!("{}", id);
    }

    Ok(())
}
