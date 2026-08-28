use crate::error::{GitletError, Result};
use crate::models::{Commit, Index};
use crate::utils::{initial_commit_timestamp, read_file_bytes, sha1_bytes, write_file_bytes};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Repo {
    pub root_dir: PathBuf,
    pub gitlet_dir: PathBuf,
}

impl Repo {
    pub fn new(root_dir: PathBuf) -> Self {
        let gitlet_dir = root_dir.join(".gitlet");
        Repo {
            root_dir,
            gitlet_dir,
        }
    }

    pub fn find() -> Result<Self> {
        let current_dir = std::env::current_dir()?;
        let mut dir = current_dir.as_path();
        loop {
            let gitlet_path = dir.join(".gitlet");
            if gitlet_path.is_dir() {
                return Ok(Repo::new(dir.to_path_buf()));
            }
            match dir.parent() {
                Some(parent) => dir = parent,
                None => return Err(GitletError::NotInitialized),
            }
        }
    }

    pub fn init(dir: &Path) -> Result<Self> {
        let gitlet_dir = dir.join(".gitlet");
        if gitlet_dir.exists() {
            return Err(GitletError::AlreadyExists);
        }

        let repo = Repo::new(dir.to_path_buf());
        fs::create_dir_all(repo.objects_dir())?;
        fs::create_dir_all(repo.heads_dir())?;

        // Create initial commit
        let initial_commit = Commit::new(
            "initial commit".to_string(),
            initial_commit_timestamp(),
            vec![],
            BTreeMap::new(),
        );
        let initial_id = repo.save_commit(&initial_commit)?;

        // Set main branch pointing to initial commit
        repo.set_branch_commit("main", &initial_id)?;

        // Set HEAD pointing to refs/heads/main
        repo.set_head_branch("main")?;

        // Save empty index
        repo.save_index(&Index::new())?;

        Ok(repo)
    }

    pub fn objects_dir(&self) -> PathBuf {
        self.gitlet_dir.join("objects")
    }

    pub fn heads_dir(&self) -> PathBuf {
        self.gitlet_dir.join("refs").join("heads")
    }

    pub fn head_file(&self) -> PathBuf {
        self.gitlet_dir.join("HEAD")
    }

    pub fn index_file(&self) -> PathBuf {
        self.gitlet_dir.join("index")
    }

    fn object_path(&self, id: &str) -> PathBuf {
        if id.len() < 2 {
            return self.objects_dir().join(id);
        }
        let (dir_part, file_part) = id.split_at(2);
        self.objects_dir().join(dir_part).join(file_part)
    }

    pub fn save_blob(&self, data: &[u8]) -> Result<String> {
        let id = sha1_bytes(data);
        let path = self.object_path(&id);
        if !path.exists() {
            write_file_bytes(&path, data)?;
        }
        Ok(id)
    }

    pub fn read_blob(&self, id: &str) -> Result<Vec<u8>> {
        let path = self.object_path(id);
        if !path.exists() {
            return Err(GitletError::Other(format!("Blob {} not found", id)));
        }
        Ok(read_file_bytes(&path)?)
    }

    pub fn save_commit(&self, commit: &Commit) -> Result<String> {
        let id = commit.id();
        let path = self.object_path(&id);
        if !path.exists() {
            let serialized = serde_json::to_vec(commit)?;
            write_file_bytes(&path, &serialized)?;
        }
        Ok(id)
    }

    pub fn read_commit(&self, id: &str) -> Result<Commit> {
        let full_id = self.find_commit_by_prefix(id)?;
        let path = self.object_path(&full_id);
        if !path.exists() {
            return Err(GitletError::NoCommitWithId);
        }
        let bytes = read_file_bytes(&path)?;
        let commit: Commit = match serde_json::from_slice(&bytes) {
            Ok(c) => c,
            Err(_) => return Err(GitletError::NoCommitWithId),
        };
        Ok(commit)
    }

    pub fn find_commit_by_prefix(&self, prefix: &str) -> Result<String> {
        if prefix.len() == 40 {
            let path = self.object_path(prefix);
            if path.exists() {
                if let Ok(bytes) = read_file_bytes(&path) {
                    if serde_json::from_slice::<Commit>(&bytes).is_ok() {
                        return Ok(prefix.to_string());
                    }
                }
            }
            return Err(GitletError::NoCommitWithId);
        }

        if prefix.len() < 6 {
            return Err(GitletError::NoCommitWithId);
        }

        let dir_part = &prefix[..2];
        let target_dir = self.objects_dir().join(dir_part);
        if !target_dir.is_dir() {
            return Err(GitletError::NoCommitWithId);
        }

        let mut matches = Vec::new();
        if let Ok(subentries) = fs::read_dir(&target_dir) {
            for subentry in subentries.flatten() {
                let file_name = subentry.file_name().to_string_lossy().to_string();
                let full_id = format!("{}{}", dir_part, file_name);
                if full_id.starts_with(prefix) {
                    if let Ok(bytes) = read_file_bytes(&subentry.path()) {
                        if serde_json::from_slice::<Commit>(&bytes).is_ok() {
                            matches.push(full_id);
                        }
                    }
                }
            }
        }

        if matches.len() == 1 {
            Ok(matches.remove(0))
        } else {
            Err(GitletError::NoCommitWithId)
        }
    }

    pub fn get_all_commits(&self) -> Result<Vec<(String, Commit)>> {
        let mut commits = Vec::new();
        let objects_dir = self.objects_dir();
        if let Ok(entries) = fs::read_dir(&objects_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = entry.file_name().to_string_lossy().to_string();
                    if let Ok(subentries) = fs::read_dir(&path) {
                        for subentry in subentries.flatten() {
                            let file_name = subentry.file_name().to_string_lossy().to_string();
                            let full_id = format!("{}{}", dir_name, file_name);
                            if let Ok(bytes) = read_file_bytes(&subentry.path()) {
                                if let Ok(commit) = serde_json::from_slice::<Commit>(&bytes) {
                                    commits.push((full_id, commit));
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(commits)
    }

    pub fn read_index(&self) -> Result<Index> {
        let path = self.index_file();
        if !path.exists() {
            return Ok(Index::new());
        }
        let bytes = read_file_bytes(&path)?;
        let index: Index = serde_json::from_slice(&bytes)?;
        Ok(index)
    }

    pub fn save_index(&self, index: &Index) -> Result<()> {
        let serialized = serde_json::to_vec_pretty(index)?;
        write_file_bytes(&self.index_file(), &serialized)?;
        Ok(())
    }

    pub fn get_head_branch_name(&self) -> Result<Option<String>> {
        let head_content = fs::read_to_string(self.head_file())?;
        let trimmed = head_content.trim();
        if let Some(rest) = trimmed.strip_prefix("ref: refs/heads/") {
            Ok(Some(rest.to_string()))
        } else {
            Ok(None)
        }
    }

    pub fn set_head_branch(&self, branch_name: &str) -> Result<()> {
        let content = format!("ref: refs/heads/{}\n", branch_name);
        fs::write(self.head_file(), content)?;
        Ok(())
    }

    pub fn get_branch_commit_id(&self, branch_name: &str) -> Result<String> {
        let branch_file = self.heads_dir().join(branch_name);
        if !branch_file.exists() {
            return Err(GitletError::NoSuchBranch);
        }
        let content = fs::read_to_string(branch_file)?;
        Ok(content.trim().to_string())
    }

    pub fn set_branch_commit(&self, branch_name: &str, commit_id: &str) -> Result<()> {
        let branch_file = self.heads_dir().join(branch_name);
        if let Some(parent) = branch_file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(branch_file, format!("{}\n", commit_id))?;
        Ok(())
    }

    pub fn branch_exists(&self, branch_name: &str) -> bool {
        self.heads_dir().join(branch_name).exists()
    }

    pub fn list_branches(&self) -> Result<Vec<String>> {
        let mut branches = Vec::new();
        if let Ok(entries) = fs::read_dir(self.heads_dir()) {
            for entry in entries.flatten() {
                if entry.path().is_file() {
                    branches.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
        branches.sort();
        Ok(branches)
    }

    pub fn delete_branch(&self, branch_name: &str) -> Result<()> {
        let branch_file = self.heads_dir().join(branch_name);
        if branch_file.exists() {
            fs::remove_file(branch_file)?;
        }
        Ok(())
    }

    pub fn get_head_commit_id(&self) -> Result<String> {
        let head_content = fs::read_to_string(self.head_file())?;
        let trimmed = head_content.trim();
        if let Some(rest) = trimmed.strip_prefix("ref: refs/heads/") {
            self.get_branch_commit_id(rest)
        } else {
            Ok(trimmed.to_string())
        }
    }

    pub fn get_head_commit(&self) -> Result<Commit> {
        let head_id = self.get_head_commit_id()?;
        self.read_commit(&head_id)
    }

    pub fn check_untracked_conflicts(
        &self,
        target_file_map: &BTreeMap<String, String>,
    ) -> Result<()> {
        let head_commit = self.get_head_commit()?;

        // Check every file in target_file_map that would be written/overwritten
        for (target_file, target_blob_id) in target_file_map {
            let working_path = self.root_dir.join(target_file);
            if working_path.exists() {
                let is_tracked = head_commit.file_map.contains_key(target_file);
                if !is_tracked {
                    let current_bytes = fs::read(&working_path).unwrap_or_default();
                    let current_blob_id = sha1_bytes(&current_bytes);
                    if &current_blob_id != target_blob_id {
                        return Err(GitletError::UntrackedFileInWay);
                    }
                }
            }
        }

        Ok(())
    }

    /// Restores the working directory to match the target commit snapshot,
    /// removing files tracked in current_commit that are absent in target_commit,
    /// and clearing the staging index.
    pub fn restore_tree_to_commit(
        &self,
        target_commit: &Commit,
        current_commit: &Commit,
    ) -> Result<()> {
        self.check_untracked_conflicts(&target_commit.file_map)?;

        // Write all files from target commit
        for (file_name, blob_id) in &target_commit.file_map {
            let blob_data = self.read_blob(blob_id)?;
            let target_path = self.root_dir.join(file_name);
            write_file_bytes(&target_path, &blob_data)?;
        }

        // Delete files tracked in current commit but absent in target commit
        for file_name in current_commit.file_map.keys() {
            if !target_commit.file_map.contains_key(file_name) {
                let path = self.root_dir.join(file_name);
                if path.exists() {
                    fs::remove_file(&path)?;
                }
            }
        }

        // Clear staging area
        let mut index = self.read_index()?;
        index.clear();
        self.save_index(&index)?;

        Ok(())
    }

    pub fn get_working_dir_files(&self) -> Result<Vec<String>> {
        let mut files = Vec::new();
        self.collect_files_recursive(&self.root_dir, "", &mut files)?;
        files.sort();
        Ok(files)
    }

    fn collect_files_recursive(
        &self,
        current_dir: &Path,
        prefix: &str,
        files: &mut Vec<String>,
    ) -> Result<()> {
        if let Ok(entries) = fs::read_dir(current_dir) {
            for entry in entries.flatten() {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name == ".gitlet" || file_name.starts_with(".git") {
                    continue;
                }
                let path = entry.path();
                let rel_path = if prefix.is_empty() {
                    file_name
                } else {
                    format!("{}/{}", prefix, file_name)
                };

                if path.is_file() {
                    files.push(rel_path);
                } else if path.is_dir() {
                    self.collect_files_recursive(&path, &rel_path, files)?;
                }
            }
        }
        Ok(())
    }
}
