use crate::error::{GitletError, Result};
use crate::models::Commit;
use crate::repo::Repo;
use crate::utils::current_timestamp;

pub fn execute(message: &str) -> Result<String> {
    execute_with_parents(message, None)
}

pub fn execute_with_parents(message: &str, second_parent: Option<&str>) -> Result<String> {
    if message.trim().is_empty() {
        return Err(GitletError::EmptyCommitMessage);
    }

    let repo = Repo::find()?;
    let mut index = repo.read_index()?;

    if index.is_empty() {
        return Err(GitletError::NoChangesAdded);
    }

    let head_commit_id = repo.get_head_commit_id()?;
    let head_commit = repo.read_commit(&head_commit_id)?;

    let mut file_map = head_commit.file_map.clone();

    // Apply additions
    for (file, blob_id) in &index.added {
        file_map.insert(file.clone(), blob_id.clone());
    }

    // Apply removals
    for file in &index.removed {
        file_map.remove(file);
    }

    let parents = match second_parent {
        Some(p2) => vec![head_commit_id, p2.to_string()],
        None => vec![head_commit_id],
    };

    let new_commit = Commit::new(message.to_string(), current_timestamp(), parents, file_map);

    let new_commit_id = repo.save_commit(&new_commit)?;

    // Update branch ref or HEAD
    if let Some(branch_name) = repo.get_head_branch_name()? {
        repo.set_branch_commit(&branch_name, &new_commit_id)?;
    } else {
        std::fs::write(repo.head_file(), format!("{}\n", new_commit_id))?;
    }

    // Clear staging area
    index.clear();
    repo.save_index(&index)?;

    Ok(new_commit_id)
}
