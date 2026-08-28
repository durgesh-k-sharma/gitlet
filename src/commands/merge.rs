use crate::dag::find_split_point;
use crate::error::{GitletError, Result};
use crate::models::Commit;
use crate::repo::Repo;
use crate::utils::{current_timestamp, write_file_bytes};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;

pub fn execute(given_branch: &str) -> Result<()> {
    let repo = Repo::find()?;

    // 1. Check index is clean
    let mut index = repo.read_index()?;
    if !index.is_empty() {
        return Err(GitletError::UncommittedChanges);
    }

    // 2. Check branch exists
    if !repo.branch_exists(given_branch) {
        return Err(GitletError::BranchDoesNotExist);
    }

    // 3. Check not merging branch with itself
    let current_branch = match repo.get_head_branch_name()? {
        Some(b) => b,
        None => return Err(GitletError::Other("HEAD is detached".to_string())),
    };

    if current_branch == given_branch {
        return Err(GitletError::CannotMergeWithSelf);
    }

    let current_commit_id = repo.get_head_commit_id()?;
    let given_commit_id = repo.get_branch_commit_id(given_branch)?;

    let current_commit = repo.read_commit(&current_commit_id)?;
    let given_commit = repo.read_commit(&given_commit_id)?;

    // 4. Find split point
    let split_id = find_split_point(&repo, &current_commit_id, &given_commit_id)?;

    // If split point is given branch -> ancestor
    if split_id == given_commit_id {
        println!("Given branch is an ancestor of the current branch.");
        return Ok(());
    }

    // If split point is current branch -> fast forward
    if split_id == current_commit_id {
        repo.restore_tree_to_commit(&given_commit, &current_commit)?;
        repo.set_branch_commit(&current_branch, &given_commit_id)?;
        println!("Current branch fast-forwarded.");
        return Ok(());
    }

    let split_commit = repo.read_commit(&split_id)?;

    // Collect all candidate files across split, current, and given commits
    let mut all_files: BTreeSet<String> = BTreeSet::new();
    for k in split_commit.file_map.keys() {
        all_files.insert(k.clone());
    }
    for k in current_commit.file_map.keys() {
        all_files.insert(k.clone());
    }
    for k in given_commit.file_map.keys() {
        all_files.insert(k.clone());
    }

    // Untracked file check: only fail if the merge action would actively overwrite or delete an untracked working file
    for file in &all_files {
        let s = split_commit.file_map.get(file);
        let c = current_commit.file_map.get(file);
        let g = given_commit.file_map.get(file);

        let will_modify_or_delete = !(c == g || (c != s && g == s));

        if will_modify_or_delete {
            let is_tracked_current = current_commit.file_map.contains_key(file);
            let path = repo.root_dir.join(file);
            if !is_tracked_current && path.exists() {
                return Err(GitletError::UntrackedFileInWay);
            }
        }
    }

    let mut conflict_occurred = false;
    let mut merged_file_map: BTreeMap<String, String> = current_commit.file_map.clone();

    for file in all_files {
        let s = split_commit.file_map.get(&file);
        let c = current_commit.file_map.get(&file);
        let g = given_commit.file_map.get(&file);

        match (s, c, g) {
            // Case 1: Present in split
            (Some(s_id), Some(c_id), Some(g_id)) => {
                if c_id == s_id && g_id == s_id {
                    // Unchanged in both -> keep current
                } else if c_id == s_id && g_id != s_id {
                    // Current unchanged, given modified -> take given
                    let data = repo.read_blob(g_id)?;
                    let path = repo.root_dir.join(&file);
                    write_file_bytes(&path, &data)?;
                    index.stage_add(file.clone(), g_id.clone());
                    merged_file_map.insert(file, g_id.clone());
                } else if c_id != s_id && g_id == s_id {
                    // Current modified, given unchanged -> keep current
                } else if c_id == g_id {
                    // Both modified the same way -> keep
                } else {
                    // Conflict: both modified differently
                    conflict_occurred = true;
                    handle_conflict(
                        &repo,
                        &file,
                        Some(c_id),
                        Some(g_id),
                        given_branch,
                        &mut index,
                        &mut merged_file_map,
                    )?;
                }
            }
            (Some(s_id), Some(c_id), None) => {
                if c_id == s_id {
                    // Given removed, current unchanged -> remove file
                    let path = repo.root_dir.join(&file);
                    if path.exists() {
                        fs::remove_file(&path)?;
                    }
                    index.stage_rm(file.clone());
                    merged_file_map.remove(&file);
                } else {
                    // Conflict: current modified, given removed
                    conflict_occurred = true;
                    handle_conflict(
                        &repo,
                        &file,
                        Some(c_id),
                        None,
                        given_branch,
                        &mut index,
                        &mut merged_file_map,
                    )?;
                }
            }
            (Some(s_id), None, Some(g_id)) => {
                if g_id == s_id {
                    // Current removed, given unchanged -> keep removed
                } else {
                    // Conflict: current removed, given modified
                    conflict_occurred = true;
                    handle_conflict(
                        &repo,
                        &file,
                        None,
                        Some(g_id),
                        given_branch,
                        &mut index,
                        &mut merged_file_map,
                    )?;
                }
            }
            (Some(_s_id), None, None) => {
                // Both removed -> keep removed
            }

            // Case 2: Absent in split
            (None, None, Some(g_id)) => {
                // Added in given -> take given
                let data = repo.read_blob(g_id)?;
                let path = repo.root_dir.join(&file);
                write_file_bytes(&path, &data)?;
                index.stage_add(file.clone(), g_id.clone());
                merged_file_map.insert(file, g_id.clone());
            }
            (None, Some(_c_id), None) => {
                // Added in current -> keep current
            }
            (None, Some(c_id), Some(g_id)) => {
                if c_id == g_id {
                    // Both added same content -> keep current
                } else {
                    // Both added different content -> conflict
                    conflict_occurred = true;
                    handle_conflict(
                        &repo,
                        &file,
                        Some(c_id),
                        Some(g_id),
                        given_branch,
                        &mut index,
                        &mut merged_file_map,
                    )?;
                }
            }
            (None, None, None) => {}
        }
    }

    // Create merge commit
    let merge_message = format!("Merged {} into {}.", given_branch, current_branch);
    let merge_commit = Commit::new(
        merge_message,
        current_timestamp(),
        vec![current_commit_id, given_commit_id],
        merged_file_map,
    );

    let merge_commit_id = repo.save_commit(&merge_commit)?;
    repo.set_branch_commit(&current_branch, &merge_commit_id)?;

    // Clear index
    index.clear();
    repo.save_index(&index)?;

    if conflict_occurred {
        println!("Encountered a merge conflict.");
    }

    Ok(())
}

fn handle_conflict(
    repo: &Repo,
    file_name: &str,
    current_blob_id: Option<&String>,
    given_blob_id: Option<&String>,
    given_branch: &str,
    index: &mut crate::models::Index,
    merged_file_map: &mut BTreeMap<String, String>,
) -> Result<()> {
    let current_bytes = match current_blob_id {
        Some(id) => repo.read_blob(id)?,
        None => Vec::new(),
    };

    let given_bytes = match given_blob_id {
        Some(id) => repo.read_blob(id)?,
        None => Vec::new(),
    };

    let mut conflict_buf = Vec::new();
    conflict_buf.extend_from_slice(b"<<<<<<< HEAD\n");
    conflict_buf.extend_from_slice(&current_bytes);
    if !current_bytes.is_empty() && !current_bytes.ends_with(b"\n") {
        conflict_buf.push(b'\n');
    }
    conflict_buf.extend_from_slice(b"=======\n");
    conflict_buf.extend_from_slice(&given_bytes);
    if !given_bytes.is_empty() && !given_bytes.ends_with(b"\n") {
        conflict_buf.push(b'\n');
    }
    conflict_buf.extend_from_slice(format!(">>>>>>> {}\n", given_branch).as_bytes());

    let file_path = repo.root_dir.join(file_name);
    write_file_bytes(&file_path, &conflict_buf)?;

    let blob_id = repo.save_blob(&conflict_buf)?;
    index.stage_add(file_name.to_string(), blob_id.clone());
    merged_file_map.insert(file_name.to_string(), blob_id);

    Ok(())
}
