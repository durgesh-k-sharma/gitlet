use crate::error::{GitletError, Result};
use crate::repo::Repo;
use std::fs;

pub fn execute(file_name: &str) -> Result<()> {
    let repo = Repo::find()?;
    let mut index = repo.read_index()?;
    let head_commit = repo.get_head_commit()?;

    let is_staged_add = index.added.contains_key(file_name);
    let is_tracked = head_commit.file_map.contains_key(file_name);

    if !is_staged_add && !is_tracked {
        return Err(GitletError::NoReasonToRemove);
    }

    if is_staged_add {
        index.unstage_add(file_name);
    }

    if is_tracked {
        index.stage_rm(file_name.to_string());
        let file_path = repo.root_dir.join(file_name);
        if file_path.exists() {
            fs::remove_file(file_path)?;
        }
    }

    repo.save_index(&index)?;
    Ok(())
}
