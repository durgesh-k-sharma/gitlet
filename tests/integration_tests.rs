use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

struct TestEnv {
    dir: TempDir,
    binary: PathBuf,
}

impl TestEnv {
    fn new() -> Self {
        let dir = TempDir::new().unwrap();
        let binary = PathBuf::from(env!("CARGO_BIN_EXE_gitlet"));
        TestEnv { dir, binary }
    }

    fn run(&self, args: &[&str]) -> (String, i32) {
        let output = Command::new(&self.binary)
            .current_dir(self.dir.path())
            .args(args)
            .output()
            .expect("Failed to execute gitlet binary");

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let exit_code = output.status.code().unwrap_or(-1);
        (stdout, exit_code)
    }

    fn write_file(&self, file_name: &str, content: &str) {
        let file_path = self.dir.path().join(file_name);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(file_path, content).unwrap();
    }

    fn write_bytes(&self, file_name: &str, content: &[u8]) {
        let file_path = self.dir.path().join(file_name);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(file_path, content).unwrap();
    }

    fn read_file(&self, file_name: &str) -> String {
        let file_path = self.dir.path().join(file_name);
        fs::read_to_string(file_path).unwrap()
    }

    fn read_bytes(&self, file_name: &str) -> Vec<u8> {
        let file_path = self.dir.path().join(file_name);
        fs::read(file_path).unwrap()
    }

    fn file_exists(&self, file_name: &str) -> bool {
        self.dir.path().join(file_name).exists()
    }
}

#[test]
fn test_init_basic_and_duplicate() {
    let env = TestEnv::new();

    // 1. Initial init
    let (out, _) = env.run(&["init"]);
    assert!(out.is_empty(), "init should output nothing on success");
    assert!(env.dir.path().join(".gitlet").is_dir());
    assert!(env.dir.path().join(".gitlet/HEAD").is_file());
    assert!(env.dir.path().join(".gitlet/refs/heads/main").is_file());

    // 2. Duplicate init
    let (out, _) = env.run(&["init"]);
    assert_eq!(
        out.trim(),
        "A Gitlet version-control system already exists in the current directory."
    );
}

#[test]
fn test_add_and_commit() {
    let env = TestEnv::new();
    env.run(&["init"]);

    // Add nonexistent file
    let (out, _) = env.run(&["add", "nonexistent.txt"]);
    assert_eq!(out.trim(), "File does not exist.");

    // Commit with empty staging
    let (out, _) = env.run(&["commit", "empty commit"]);
    assert_eq!(out.trim(), "No changes added to the commit.");

    // Add valid file and commit
    env.write_file("hello.txt", "hello world");
    env.run(&["add", "hello.txt"]);

    // Commit with empty message
    let (out, _) = env.run(&["commit", ""]);
    assert_eq!(out.trim(), "Please enter a commit message.");

    // Commit successfully
    let (out, _) = env.run(&["commit", "Added hello.txt"]);
    assert!(out.is_empty());

    // Re-adding identical file should not stage anything
    env.run(&["add", "hello.txt"]);
    let (out, _) = env.run(&["commit", "no changes"]);
    assert_eq!(out.trim(), "No changes added to the commit.");
}

#[test]
fn test_rm() {
    let env = TestEnv::new();
    env.run(&["init"]);

    env.write_file("a.txt", "content a");
    env.write_file("b.txt", "content b");
    env.run(&["add", "a.txt"]);
    env.run(&["commit", "add a"]);

    // Rm untracked/unstaged
    let (out, _) = env.run(&["rm", "b.txt"]);
    assert_eq!(out.trim(), "No reason to remove the file.");

    // Stage b, then unstage with rm
    env.run(&["add", "b.txt"]);
    let (out, _) = env.run(&["status"]);
    assert!(out.contains("b.txt"));

    env.run(&["rm", "b.txt"]);
    let (out, _) = env.run(&["status"]);
    assert!(!out.contains("=== Staged Files ===\nb.txt"));

    // Rm tracked file a.txt
    env.run(&["rm", "a.txt"]);
    assert!(!env.file_exists("a.txt"));

    let (out, _) = env.run(&["status"]);
    assert!(out.contains("=== Removed Files ===\na.txt"));

    env.run(&["commit", "remove a"]);
    let (out, _) = env.run(&["status"]);
    assert!(!out.contains("a.txt"));
}

#[test]
fn test_log_find_status() {
    let env = TestEnv::new();
    env.run(&["init"]);

    env.write_file("file1.txt", "v1");
    env.run(&["add", "file1.txt"]);
    env.run(&["commit", "first message"]);

    env.write_file("file2.txt", "v2");
    env.run(&["add", "file2.txt"]);
    env.run(&["commit", "second message"]);

    // Create untracked file
    env.write_file("untracked.txt", "not tracked");

    // Log
    let (log_out, _) = env.run(&["log"]);
    assert!(log_out.contains("second message"));
    assert!(log_out.contains("first message"));
    assert!(log_out.contains("initial commit"));

    // Find
    let (find_out, _) = env.run(&["find", "first message"]);
    let first_id = find_out.trim();
    assert_eq!(first_id.len(), 40);

    let (find_missing, _) = env.run(&["find", "missing message"]);
    assert_eq!(find_missing.trim(), "Found no commit with that message.");

    // Status
    let (status_out, _) = env.run(&["status"]);
    assert!(status_out.contains("=== Branches ===\n*main"));
    assert!(status_out.contains("=== Untracked Files ===\nuntracked.txt"));
}

#[test]
fn test_branch_and_rm_branch() {
    let env = TestEnv::new();
    env.run(&["init"]);

    env.run(&["branch", "feature"]);
    let (status_out, _) = env.run(&["status"]);
    assert!(status_out.contains("*main"));
    assert!(status_out.contains("feature"));

    // Duplicate branch
    let (out, _) = env.run(&["branch", "feature"]);
    assert_eq!(out.trim(), "A branch with that name already exists.");

    // Cannot remove current branch
    let (out, _) = env.run(&["rm-branch", "main"]);
    assert_eq!(out.trim(), "Cannot remove the current branch.");

    // Remove feature branch
    let (out, _) = env.run(&["rm-branch", "feature"]);
    assert!(out.is_empty());

    // Remove nonexistent branch
    let (out, _) = env.run(&["rm-branch", "feature"]);
    assert_eq!(out.trim(), "A branch with that name does not exist.");
}

#[test]
fn test_checkout_forms_and_reset() {
    let env = TestEnv::new();
    env.run(&["init"]);

    env.write_file("file.txt", "version 1");
    env.run(&["add", "file.txt"]);
    env.run(&["commit", "commit 1"]);

    let (find_out, _) = env.run(&["find", "commit 1"]);
    let c1_id = find_out.trim().to_string();

    env.write_file("file.txt", "version 2");
    env.run(&["add", "file.txt"]);
    env.run(&["commit", "commit 2"]);

    assert_eq!(env.read_file("file.txt"), "version 2");

    // Checkout file from HEAD after modifying working tree
    env.write_file("file.txt", "modified uncommitted");
    env.run(&["checkout", "--", "file.txt"]);
    assert_eq!(env.read_file("file.txt"), "version 2");

    // Checkout file from past commit (using prefix >= 6 chars)
    let prefix = &c1_id[..8];
    env.run(&["checkout", prefix, "--", "file.txt"]);
    assert_eq!(env.read_file("file.txt"), "version 1");

    // Branch switching checkout
    env.run(&["branch", "other"]);
    env.run(&["checkout", "other"]);

    let (status_out, _) = env.run(&["status"]);
    assert!(status_out.contains("*other"));

    // Reset to commit 1
    env.run(&["reset", prefix]);
    assert_eq!(env.read_file("file.txt"), "version 1");
}

#[test]
fn test_merge_fast_forward_and_ancestor() {
    let env = TestEnv::new();
    env.run(&["init"]);

    env.write_file("a.txt", "init a");
    env.run(&["add", "a.txt"]);
    env.run(&["commit", "initial a"]);

    env.run(&["branch", "b1"]);
    env.run(&["checkout", "b1"]);

    env.write_file("b.txt", "branch file");
    env.run(&["add", "b.txt"]);
    env.run(&["commit", "commit on b1"]);

    // Switch back to main and merge b1 (split point is main -> fast forward)
    env.run(&["checkout", "main"]);
    let (out, _) = env.run(&["merge", "b1"]);
    assert_eq!(out.trim(), "Current branch fast-forwarded.");
    assert_eq!(env.read_file("b.txt"), "branch file");

    // Switch to b1 and merge main (split point is main/b1 -> ancestor)
    env.run(&["checkout", "b1"]);
    let (out, _) = env.run(&["merge", "main"]);
    assert_eq!(
        out.trim(),
        "Given branch is an ancestor of the current branch."
    );
}

#[test]
fn test_merge_three_way_clean() {
    let env = TestEnv::new();
    env.run(&["init"]);

    env.write_file("base.txt", "base");
    env.write_file("split_mod.txt", "split");
    env.run(&["add", "base.txt"]);
    env.run(&["add", "split_mod.txt"]);
    env.run(&["commit", "base commit"]);

    env.run(&["branch", "feature"]);

    // Main modifies main_file and split_mod
    env.write_file("main_only.txt", "from main");
    env.run(&["add", "main_only.txt"]);
    env.run(&["commit", "main change"]);

    // Feature modifies split_mod and adds feature_only
    env.run(&["checkout", "feature"]);
    env.write_file("split_mod.txt", "split modified by feature");
    env.write_file("feature_only.txt", "from feature");
    env.run(&["add", "split_mod.txt"]);
    env.run(&["add", "feature_only.txt"]);
    env.run(&["commit", "feature change"]);

    // Switch to main and merge feature
    env.run(&["checkout", "main"]);
    let (out, _) = env.run(&["merge", "feature"]);
    assert!(
        out.is_empty(),
        "Clean merge should produce no error/conflict message"
    );

    assert_eq!(env.read_file("base.txt"), "base");
    assert_eq!(env.read_file("main_only.txt"), "from main");
    assert_eq!(env.read_file("feature_only.txt"), "from feature");
    assert_eq!(env.read_file("split_mod.txt"), "split modified by feature");

    // Check merge commit log
    let (log_out, _) = env.run(&["log"]);
    assert!(log_out.contains("Merge:"));
    assert!(log_out.contains("Merged feature into main."));
}

#[test]
fn test_merge_conflict() {
    let env = TestEnv::new();
    env.run(&["init"]);

    env.write_file("shared.txt", "line 1\n");
    env.run(&["add", "shared.txt"]);
    env.run(&["commit", "base commit"]);

    env.run(&["branch", "feature"]);

    // Main changes shared.txt
    env.write_file("shared.txt", "line 1 modified by main\n");
    env.run(&["add", "shared.txt"]);
    env.run(&["commit", "main modification"]);

    // Feature changes shared.txt differently
    env.run(&["checkout", "feature"]);
    env.write_file("shared.txt", "line 1 modified by feature\n");
    env.run(&["add", "shared.txt"]);
    env.run(&["commit", "feature modification"]);

    // Merge feature into main
    env.run(&["checkout", "main"]);
    let (out, _) = env.run(&["merge", "feature"]);
    assert_eq!(out.trim(), "Encountered a merge conflict.");

    let content = env.read_file("shared.txt");
    let expected = "<<<<<<< HEAD\nline 1 modified by main\n=======\nline 1 modified by feature\n>>>>>>> feature\n";
    assert_eq!(content, expected);

    // Merge commit should have been made
    let (log_out, _) = env.run(&["log"]);
    assert!(log_out.contains("Merge:"));
    assert!(log_out.contains("Merged feature into main."));
}

#[test]
fn test_untracked_file_conflict_detection() {
    let env = TestEnv::new();
    env.run(&["init"]);

    env.write_file("tracked.txt", "v1");
    env.run(&["add", "tracked.txt"]);
    env.run(&["commit", "c1"]);

    env.run(&["branch", "other"]);
    env.run(&["checkout", "other"]);

    env.write_file("untracked_in_main.txt", "from other");
    env.run(&["add", "untracked_in_main.txt"]);
    env.run(&["commit", "c2"]);

    // Switch to main
    env.run(&["checkout", "main"]);

    // Create an untracked file with same name in main
    env.write_file("untracked_in_main.txt", "dirty untracked content");

    // Attempting to checkout other branch should fail
    let (out, _) = env.run(&["checkout", "other"]);
    assert_eq!(
        out.trim(),
        "There is an untracked file in the way; delete it, or add and commit it first."
    );

    // Attempting to merge other branch should fail
    let (out, _) = env.run(&["merge", "other"]);
    assert_eq!(
        out.trim(),
        "There is an untracked file in the way; delete it, or add and commit it first."
    );
}

#[test]
fn test_merge_error_validations() {
    let env = TestEnv::new();
    env.run(&["init"]);

    // Merging nonexistent branch
    let (out, _) = env.run(&["merge", "nonexistent"]);
    assert_eq!(out.trim(), "A branch with that name does not exist.");

    // Merging branch with itself
    let (out, _) = env.run(&["merge", "main"]);
    assert_eq!(out.trim(), "Cannot merge a branch with itself.");

    // Merging with uncommitted changes
    env.write_file("f.txt", "hello");
    env.run(&["add", "f.txt"]);
    let (out, _) = env.run(&["merge", "main"]);
    assert_eq!(out.trim(), "You have uncommitted changes.");
}

#[test]
fn test_global_log() {
    let env = TestEnv::new();
    env.run(&["init"]);

    env.write_file("f1.txt", "1");
    env.run(&["add", "f1.txt"]);
    env.run(&["commit", "branch1 commit"]);

    env.run(&["branch", "b2"]);
    env.run(&["checkout", "b2"]);

    env.write_file("f2.txt", "2");
    env.run(&["add", "f2.txt"]);
    env.run(&["commit", "branch2 commit"]);

    let (out, _) = env.run(&["global-log"]);
    assert!(out.contains("initial commit"));
    assert!(out.contains("branch1 commit"));
    assert!(out.contains("branch2 commit"));
}

#[test]
fn test_merge_binary_conflict_safety() {
    let env = TestEnv::new();
    env.run(&["init"]);

    // Binary file with non-utf8 bytes (e.g. 0xFF, 0xFE, 0x00)
    let base_bytes = vec![0x00, 0x01, 0xFE, 0xFF, b'\n'];
    let main_bytes = vec![0x00, 0x01, 0x02, 0xFE, 0xFF, b'\n'];
    let feat_bytes = vec![0x00, 0x01, 0x03, 0xFE, 0xFF, b'\n'];

    env.write_bytes("bin.dat", &base_bytes);
    env.run(&["add", "bin.dat"]);
    env.run(&["commit", "base binary"]);

    env.run(&["branch", "feature"]);

    // Modify in main
    env.write_bytes("bin.dat", &main_bytes);
    env.run(&["add", "bin.dat"]);
    env.run(&["commit", "main binary"]);

    // Modify in feature
    env.run(&["checkout", "feature"]);
    env.write_bytes("bin.dat", &feat_bytes);
    env.run(&["add", "bin.dat"]);
    env.run(&["commit", "feature binary"]);

    // Merge
    env.run(&["checkout", "main"]);
    let (out, _) = env.run(&["merge", "feature"]);
    assert_eq!(out.trim(), "Encountered a merge conflict.");

    let result_bytes = env.read_bytes("bin.dat");
    assert!(result_bytes.starts_with(b"<<<<<<< HEAD\n"));
    assert!(result_bytes.windows(8).any(|w| w == b"=======\n"));
    assert!(result_bytes.ends_with(b">>>>>>> feature\n"));
}
