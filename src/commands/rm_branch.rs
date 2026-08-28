use crate::error::{GitletError, Result};
use crate::repo::Repo;

pub fn execute(branch_name: &str) -> Result<()> {
    let repo = Repo::find()?;

    if !repo.branch_exists(branch_name) {
        return Err(GitletError::BranchDoesNotExist);
    }

    if let Ok(Some(current_branch)) = repo.get_head_branch_name() {
        if current_branch == branch_name {
            return Err(GitletError::CannotRemoveCurrentBranch);
        }
    }

    repo.delete_branch(branch_name)?;
    Ok(())
}
