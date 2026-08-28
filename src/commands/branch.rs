use crate::error::{GitletError, Result};
use crate::repo::Repo;

pub fn execute(branch_name: &str) -> Result<()> {
    let repo = Repo::find()?;

    if repo.branch_exists(branch_name) {
        return Err(GitletError::BranchAlreadyExists);
    }

    let head_commit_id = repo.get_head_commit_id()?;
    repo.set_branch_commit(branch_name, &head_commit_id)?;

    Ok(())
}
