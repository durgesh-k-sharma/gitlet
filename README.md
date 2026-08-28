# Gitlet

Gitlet is a small content-addressed version control system written in Rust, based on Git and the UC Berkeley CS61B Gitlet specification.

It stores project history as an immutable directed acyclic graph of snapshots on disk. It handles hashing, object storage, staging, branching, and three-way merging with lowest common ancestor detection.

---

## Architecture and disk layout

Gitlet stores repository state in a `.gitlet` directory in the project root:

```
.gitlet/
├── objects/              # Blobs and commits stored by SHA-1 hash
│   ├── ab/
│   │   └── cdef123...   # First 2 hex chars for directory, rest for filename
│   └── ...
├── refs/
│   └── heads/            # Branch files containing commit IDs
│       └── main
├── HEAD                  # Points to the active branch ref
└── index                 # Staging area for additions and removals
```

### Core concepts

- **Blobs.** File snapshots indexed by their SHA-1 hash (`sha1(file_bytes)`). Identical files share the same blob.
- **Commits.** Snapshots containing a log message, timestamp, parent commit IDs, and a map of filenames to blob IDs. A commit ID is the SHA-1 hash of its serialized fields.
- **Commit graph.** Commits point to their parents. Normal commits have one parent. Merge commits have two parents. The initial commit has none.
- **Index.** The staging area tracks files staged for addition and files marked for removal before you commit.
- **Three-way merge.** Gitlet computes the lowest common ancestor split point between two branches, checks an 8-case difference table, and inserts standard conflict markers when changes overlap.

---

## Commands

### Repository setup

```bash
# Initialize a new repository
./gitlet init
```

### Staging and commits

```bash
# Stage a file
./gitlet add hello.txt

# Commit staged changes
./gitlet commit "Add hello.txt"

# Unstage a file or stage a tracked file for removal
./gitlet rm hello.txt

# Check branch, staged, and untracked files
./gitlet status
```

### History and search

```bash
# View commit history from HEAD back to the initial commit
./gitlet log

# View all commits ever made across all branches
./gitlet global-log

# Find commits matching an exact commit message
./gitlet find "Add hello.txt"
```

### Branches, checkout, and reset

```bash
# Create a new branch
./gitlet branch feature

# Switch branches
./gitlet checkout feature

# Restore a file from HEAD
./gitlet checkout -- hello.txt

# Restore a file from a specific commit (supports prefixes >= 6 chars)
./gitlet checkout a1b2c3 -- hello.txt

# Reset working directory and current branch to a commit
./gitlet reset a1b2c3

# Delete a branch pointer
./gitlet rm-branch feature
```

### Merging

```bash
# Merge a branch into the current branch
./gitlet merge feature
```

- **Fast-forward:** If the split point is current HEAD, Gitlet moves the branch pointer forward to the target commit.
- **Ancestor:** If the target branch is already an ancestor of HEAD, Gitlet exits without changes.
- **Three-way merge:** Applies changes made on both branches relative to their split point.
- **Conflicts:** When both branches modify or delete the same file in conflicting ways, Gitlet writes conflict markers, stages the file, and makes a merge commit:

```
<<<<<<< HEAD
contents from current branch
=======
contents from target branch
>>>>>>> feature
```

---

## Building and testing

### Build

```bash
cargo build --release
```

The compiled binary is at `target/release/gitlet`. You can also run `./gitlet` directly through the wrapper script.

### Tests

Run the full integration test suite:

```bash
cargo test
```

---

## License

MIT
