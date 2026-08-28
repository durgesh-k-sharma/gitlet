use crate::utils::sha1_bytes;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Commit {
    pub message: String,
    pub timestamp: String,
    pub parents: Vec<String>,
    pub file_map: BTreeMap<String, String>, // filename -> blob SHA-1 ID
}

impl Commit {
    pub fn new(
        message: String,
        timestamp: String,
        parents: Vec<String>,
        file_map: BTreeMap<String, String>,
    ) -> Self {
        Commit {
            message,
            timestamp,
            parents,
            file_map,
        }
    }

    pub fn id(&self) -> String {
        let serialized = serde_json::to_vec(self).expect("Failed to serialize commit");
        sha1_bytes(&serialized)
    }

    pub fn is_initial(&self) -> bool {
        self.parents.is_empty()
    }

    pub fn is_merge(&self) -> bool {
        self.parents.len() == 2
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Index {
    pub added: BTreeMap<String, String>, // filename -> blob_id
    pub removed: BTreeSet<String>,       // filenames staged for removal
}

impl Index {
    pub fn new() -> Self {
        Index::default()
    }

    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty()
    }

    pub fn stage_add(&mut self, file: String, blob_id: String) {
        self.removed.remove(&file);
        self.added.insert(file, blob_id);
    }

    pub fn stage_rm(&mut self, file: String) {
        self.added.remove(&file);
        self.removed.insert(file);
    }

    pub fn unstage_add(&mut self, file: &str) {
        self.added.remove(file);
    }

    pub fn unstage_rm(&mut self, file: &str) {
        self.removed.remove(file);
    }

    pub fn unstage(&mut self, file: &str) {
        self.added.remove(file);
        self.removed.remove(file);
    }

    pub fn clear(&mut self) {
        self.added.clear();
        self.removed.clear();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blob {
    pub id: String,
    pub data: Vec<u8>,
}

impl Blob {
    pub fn new(data: Vec<u8>) -> Self {
        let id = sha1_bytes(&data);
        Blob { id, data }
    }
}
