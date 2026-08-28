use crate::error::Result;
use crate::repo::Repo;

pub fn execute() -> Result<()> {
    let repo = Repo::find()?;
    let current_branch = repo.get_head_branch_name()?.unwrap_or_default();
    let branches = repo.list_branches()?;
    let index = repo.read_index()?;
    let head_commit = repo.get_head_commit()?;

    // === Branches ===
    println!("=== Branches ===");
    for branch in &branches {
        if branch == &current_branch {
            println!("*{}", branch);
        } else {
            println!("{}", branch);
        }
    }
    println!();

    // === Staged Files ===
    println!("=== Staged Files ===");
    let mut staged_files: Vec<_> = index.added.keys().collect();
    staged_files.sort();
    for file in staged_files {
        println!("{}", file);
    }
    println!();

    // === Removed Files ===
    println!("=== Removed Files ===");
    let mut removed_files: Vec<_> = index.removed.iter().collect();
    removed_files.sort();
    for file in removed_files {
        println!("{}", file);
    }
    println!();

    // === Modifications Not Staged For Commit ===
    println!("=== Modifications Not Staged For Commit ===");
    println!();

    // === Untracked Files ===
    println!("=== Untracked Files ===");
    let working_files = repo.get_working_dir_files()?;
    let mut untracked_files = Vec::new();
    for file in working_files {
        let is_tracked = head_commit.file_map.contains_key(&file);
        let is_staged_add = index.added.contains_key(&file);
        if !is_tracked && !is_staged_add {
            untracked_files.push(file);
        }
    }
    untracked_files.sort();
    for file in untracked_files {
        println!("{}", file);
    }

    Ok(())
}
