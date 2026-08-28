use crate::error::{GitletError, Result};
use crate::repo::Repo;
use crate::utils::write_file_bytes;

pub fn checkout_file_head(file_name: &str) -> Result<()> {
    let repo = Repo::find()?;
    let head_commit = repo.get_head_commit()?;
    checkout_file_from_commit(&repo, &head_commit, file_name)
}

pub fn checkout_file_commit(commit_id_prefix: &str, file_name: &str) -> Result<()> {
    let repo = Repo::find()?;
    let commit = repo.read_commit(commit_id_prefix)?;
    checkout_file_from_commit(&repo, &commit, file_name)
}

fn checkout_file_from_commit(
    repo: &Repo,
    commit: &crate::models::Commit,
    file_name: &str,
) -> Result<()> {
    let blob_id = match commit.file_map.get(file_name) {
        Some(id) => id,
        None => return Err(GitletError::NoFileInCommit),
    };

    let blob_data = repo.read_blob(blob_id)?;
    let target_path = repo.root_dir.join(file_name);
    write_file_bytes(&target_path, &blob_data)?;

    Ok(())
}

pub fn checkout_branch(branch_name: &str) -> Result<()> {
    let repo = Repo::find()?;

    if !repo.branch_exists(branch_name) {
        return Err(GitletError::NoSuchBranch);
    }

    if let Ok(Some(current_branch)) = repo.get_head_branch_name() {
        if current_branch == branch_name {
            return Err(GitletError::NoNeedToCheckoutCurrentBranch);
        }
    }

    let target_commit_id = repo.get_branch_commit_id(branch_name)?;
    let target_commit = repo.read_commit(&target_commit_id)?;
    let current_commit = repo.get_head_commit()?;

    // Restore working tree and clear index
    repo.restore_tree_to_commit(&target_commit, &current_commit)?;

    // Point HEAD to the new branch
    repo.set_head_branch(branch_name)?;

    Ok(())
}
