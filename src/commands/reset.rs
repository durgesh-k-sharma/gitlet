use crate::error::Result;
use crate::repo::Repo;
use std::fs;

pub fn execute(commit_id_prefix: &str) -> Result<()> {
    let repo = Repo::find()?;
    let full_commit_id = repo.find_commit_by_prefix(commit_id_prefix)?;
    let commit = repo.read_commit(&full_commit_id)?;
    let current_commit = repo.get_head_commit()?;

    // Restore working tree and clear index
    repo.restore_tree_to_commit(&commit, &current_commit)?;

    // Move current branch to commit
    if let Some(branch_name) = repo.get_head_branch_name()? {
        repo.set_branch_commit(&branch_name, &full_commit_id)?;
    } else {
        fs::write(repo.head_file(), format!("{}\n", full_commit_id))?;
    }

    Ok(())
}
