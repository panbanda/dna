# DNA Versioning and Storage Management

DNA uses LanceDB for storage, which maintains version history of all changes. This enables time-travel queries and artifact history tracking.

## Configuration

```toml
# .dna/config.toml
[storage]
auto_prune = false  # default: preserve history for time travel
```

- `auto_prune = false` (default): Preserve all versions for history queries
- `auto_prune = true`: Automatically compact after mutations (saves disk space)

---

## Commands

### dna prune

Manually compact the database and remove old versions.

```
dna prune [OPTIONS]

Options:
    --keep-versions <N>    Keep last N versions [default: 1]
    --older-than <DAYS>    Remove versions older than N days
    --dry-run              Show what would be pruned without doing it

Examples:
    dna prune                      # Compact, keep only current version
    dna prune --keep-versions 10   # Keep last 10 versions
    dna prune --older-than 30      # Remove versions older than 30 days
    dna prune --dry-run            # Preview space savings
```

### dna versions

List all database versions with timestamps.

```
dna versions [OPTIONS]

Options:
    --limit <N>    Show last N versions [default: 20]

Output:
    Version  Timestamp             Size     Operation
    5        2024-01-15 10:30:00   12 KB    Update artifact abc123
    4        2024-01-15 10:25:00   11 KB    Add artifact def456
    3        2024-01-15 10:20:00   8 KB     Delete artifact old789
    ...
```

### dna history

Show version history for a specific artifact.

```
dna history <ID> [OPTIONS]

Arguments:
    <ID>    Artifact ID

Options:
    --limit <N>    Show last N versions [default: 10]

Output:
    Version  Timestamp             Changes
    5        2024-01-15 10:30:00   content, metadata.status
    3        2024-01-14 15:00:00   metadata.priority
    1        2024-01-14 09:00:00   (created)
```

### dna get --version

Retrieve an artifact at a specific version.

```
dna get <ID> --version <N>

Arguments:
    <ID>    Artifact ID

Options:
    --version <N>    Database version number

Example:
    dna get abc123 --version 3
```

### dna diff

Compare artifact versions.

```
dna diff <ID> [OPTIONS]

Arguments:
    <ID>    Artifact ID

Options:
    --from <N>    Start version [default: previous]
    --to <N>      End version [default: current]

Example:
    dna diff abc123                  # Compare current vs previous
    dna diff abc123 --from 1 --to 5  # Compare specific versions

Output:
    --- version 1 (2024-01-14 09:00:00)
    +++ version 5 (2024-01-15 10:30:00)

    kind: intent
    content:
    - Users can log in with email
    + Users can log in with email and password

    metadata:
    + status: approved
```

---

## Storage Details

LanceDB uses an append-only columnar format:

```
.dna/db/artifacts.lance/
├── _transactions/     # Write-ahead log for ACID
├── _versions/         # Version manifests (time travel)
│   ├── 1.manifest
│   ├── 2.manifest
│   └── ...
└── data/              # Columnar data files
    └── *.lance
```

**How versioning works:**

1. Each mutation (insert/update/delete) creates a new version
2. Version manifests point to data files
3. Old data files are retained until pruned
4. Queries default to latest version unless specified

**Storage growth:**

- Updates create new data files (old content preserved)
- Deletes mark rows as deleted (data remains until compaction)
- Without pruning, storage grows with each mutation

**When to prune:**

- Disk space constrained environments
- After bulk imports or migrations
- When history is tracked externally (e.g., git)

**When to preserve history:**

- Audit requirements
- Debugging content changes
- Understanding requirement evolution
- Environments without git

---

## Implementation Notes

### LanceDB API

```rust
// Open at specific version
let table = db.open_table("artifacts").await?;
let old_table = table.checkout_version(version).await?;

// Compact and cleanup
table.compact_files().await?;  // Merge small files
table.cleanup().await?;         // Remove unreferenced data

// Get version info
let version = table.version().await?;
```

### Auto-prune on mutation

When `auto_prune = true`:

```rust
impl ArtifactService {
    async fn insert(&self, artifact: &Artifact) -> Result<()> {
        self.db.insert(artifact).await?;

        if self.config.storage.auto_prune.unwrap_or(false) {
            self.db.compact().await?;
        }
        Ok(())
    }
}
```

### Version metadata

Each version should track:
- Version number
- Timestamp
- Operation type (insert/update/delete)
- Affected artifact IDs
- User/agent (if available)

This metadata enables meaningful history display without scanning all data.
