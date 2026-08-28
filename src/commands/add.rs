use crate::error::{GitletError, Result};
use crate::repo::Repo;
use std::fs;

pub fn execute(file_name: &str) -> Result<()> {
    let repo = Repo::find()?;
    let file_path = repo.root_dir.join(file_name);

    if !file_path.exists() || !file_path.is_file() {
        return Err(GitletError::FileNotFound);
    }

    let file_bytes = fs::read(&file_path)?;
    let mut index = repo.read_index()?;
    let head_commit = repo.get_head_commit()?;

    let blob_id = crate::utils::sha1_bytes(&file_bytes);

    if let Some(tracked_blob_id) = head_commit.file_map.get(file_name) {
        if tracked_blob_id == &blob_id {
            // File is identical to version in current commit.
            // Do not stage, and remove from staging if it was staged.
            index.unstage(file_name);
            repo.save_index(&index)?;
            return Ok(());
        }
    }

    // Save blob to objects
    repo.save_blob(&file_bytes)?;

    // Stage for addition (this also removes from staged removals)
    index.stage_add(file_name.to_string(), blob_id);
    repo.save_index(&index)?;

    Ok(())
}
