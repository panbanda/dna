# DNA CLI Reference

Complete reference for the DNA command-line interface. Designed for both human users and LLM agents.

## Quick Reference

```bash
# Initialize a project
dna init

# Register artifact kinds
dna kind add <name> <description>
dna kind list

# Add artifacts
dna add <kind> <content> [--label key=value] [--context "..."]

# Search and retrieve
dna search <query> [--kind <kind>] [--limit <n>]
dna get <id>
dna list [--kind <kind>] [--label key=value]

# Update and remove
dna update <id> [--content "..."] [--label key=value] [--context "..."]
dna remove <id>

# Maintenance
dna reindex [--all] [--content] [--context]
```

---

## Commands

### dna init

Initialize a new DNA project in the current directory.

```
dna init [OPTIONS]

Options:
      --intent-flow    Initialize with intent-flow kinds (intent, contract, algorithm, evaluation)
  -h, --help           Print help
```

Creates a `.dna/` directory with:
- `config.toml` - Project configuration
- `artifacts.lance` - Vector database (created on first artifact)

**Examples:**

```bash
# Basic initialization
dna init

# With predefined intent-flow kinds
dna init --intent-flow
```

---

### dna kind

Manage artifact kinds. Kinds categorize artifacts and enable filtered search.

#### dna kind add

Register a new artifact kind.

```
dna kind add <NAME> <DESCRIPTION>

Arguments:
  <NAME>         Kind name. Will be slugified to kebab-case.
                 Must be 2-64 characters, lowercase alphanumeric and hyphens.
                 Cannot be a reserved word (all, any, artifact, config, etc).

  <DESCRIPTION>  Human-readable description of what this kind contains.
                 Used by LLMs to understand when to use this kind.
                 Be specific about the purpose and typical content.
```

**Examples:**

```bash
# Register an intent kind for user goals
dna kind add intent "High-level user goals and requirements that describe what the system should accomplish"

# Register a contract kind for API definitions
dna kind add contract "API contracts, interface definitions, and data schemas that define system boundaries"

# Register an algorithm kind for implementation logic
dna kind add algorithm "Implementation algorithms, business logic, and computational procedures"

# Register an evaluation kind for test criteria
dna kind add evaluation "Test criteria, acceptance conditions, and validation rules"
```

**Output:**

```
Added kind: intent
  Description: High-level user goals and requirements...

You can now use:
  dna add intent <content>           # add an intent artifact
  dna search <query> --kind intent   # search intent artifacts
  dna list --kind intent             # list intent artifacts

API endpoint:  POST /api/v1/kinds/intent/artifacts
MCP tool:      dna_intent_search, dna_intent_add
```

#### dna kind list

List all registered kinds.

```
dna kind list
```

**Output:**

```
Registered kinds (4):
  intent - High-level user goals and requirements...
  contract - API contracts, interface definitions...
  algorithm - Implementation algorithms, business logic...
  evaluation - Test criteria, acceptance conditions...
```

#### dna kind show

Show details for a specific kind.

```
dna kind show <SLUG>

Arguments:
  <SLUG>    Kind slug (e.g., "intent", "api-contract")
```

**Example:**

```bash
dna kind show intent
```

#### dna kind remove

Remove a registered kind.

```
dna kind remove <SLUG> [OPTIONS]

Arguments:
  <SLUG>    Kind slug to remove

Options:
  -f, --force    Remove without confirmation warning
```

Note: Removing a kind does not delete existing artifacts of that kind. Those artifacts become orphaned and may not appear in kind-filtered searches.

---

### dna add

Add a new artifact to the knowledge base.

```
dna add <KIND> <CONTENT> [OPTIONS]

Arguments:
  <KIND>      Artifact kind. Must be registered via 'dna kind add'.
              Use 'dna kind list' to see available kinds.

  <CONTENT>   Artifact content. The full text to be embedded and stored.
              For multi-line content, use quotes or heredoc syntax.

Options:
  -l, --label <KEY=VALUE>
              Label as key=value pair. Can be repeated for multiple labels.
              Use for filtering, organization, and metadata.
              Keys and values are case-sensitive strings.

              Examples:
                --label domain=auth
                --label priority=high
                --label owner=alice

  -c, --context <TEXT>
              Additional context for improved semantic retrieval.
              Describe relationships, domain concepts, or purpose.
              Gets its own embedding for context-aware search.

              Use context to help find this artifact when:
              - The content doesn't mention relevant terms directly
              - Related concepts should surface this artifact
              - Domain knowledge aids discovery

              Examples:
                --context "Part of the authentication system. Related to GDPR compliance."
                --context "Implements the payment flow. Depends on user-service and billing-api."

      --name <NAME>
              Optional name slug for human-readable identification.

      --format <FORMAT>
              Content format. Affects syntax highlighting and validation.
              [default: markdown]
              [possible values: markdown, yaml, json, openapi, text]

  -h, --help  Print help
```

**Examples:**

```bash
# Basic artifact
dna add intent "Users should be able to log in with email and password"

# With labels for filtering
dna add intent "Users should be able to reset their password via email" \
  --label domain=auth \
  --label priority=high

# With context for improved retrieval
dna add algorithm "Validate JWT token and extract claims" \
  --label domain=auth \
  --context "Part of the authentication middleware. Runs on every API request. Must handle expired tokens gracefully."

# Multi-line content with heredoc
dna add contract "$(cat <<'EOF'
## User API

### POST /users
Creates a new user account.

Request:
- email: string (required)
- password: string (required, min 8 chars)

Response:
- 201: User created
- 400: Validation error
- 409: Email already exists
EOF
)" --format markdown --label domain=users

# With all options
dna add evaluation "Login should complete within 500ms under normal load" \
  --name login-performance \
  --label domain=auth \
  --label type=performance \
  --context "Non-functional requirement. Measured at p95. Excludes external IdP latency."
```

**Output:**

```json
Added artifact: abc123defg
{
  "id": "abc123defg",
  "kind": "intent",
  "content": "Users should be able to log in...",
  "format": "markdown",
  "metadata": {
    "domain": "auth",
    "priority": "high"
  },
  "context": "Part of the authentication system...",
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T10:30:00Z"
}
```

**Errors:**

```
Error: Content exceeds maximum of 512 tokens (got 847).
Reduce content length or configure a model with a larger context window.
```

```
Error: Kind 'unknown-kind' is not registered.
Use 'dna kind list' to see available kinds, or 'dna kind add' to register a new one.
```

---

### dna get

Retrieve an artifact by ID.

```
dna get <ID>

Arguments:
  <ID>    Artifact ID (10-character alphanumeric string)
```

**Example:**

```bash
dna get abc123defg
```

---

### dna update

Update an existing artifact.

```
dna update <ID> [OPTIONS]

Arguments:
  <ID>    Artifact ID to update

Options:
      --content <TEXT>
              New content. Replaces existing content entirely.
              Triggers re-embedding.

      --kind <KIND>
              Change artifact kind. Must be a registered kind.

      --name <NAME>
              Change artifact name.

  -l, --label <KEY=VALUE>
              Add or update a label. Can be repeated.
              To remove a label, use empty value: --label key=

              Examples:
                --label status=done       # add/update label
                --label draft=            # remove 'draft' label

  -c, --context <TEXT>
              New context. Replaces existing context entirely.
              Triggers re-embedding of context.
              Use empty string to remove: --context ""

  -h, --help  Print help
```

**Examples:**

```bash
# Update content
dna update abc123defg --content "Updated requirement text"

# Add a label
dna update abc123defg --label status=approved

# Remove a label (empty value)
dna update abc123defg --label draft=

# Update context
dna update abc123defg --context "Now part of the v2 auth system"

# Remove context
dna update abc123defg --context ""

# Multiple changes at once
dna update abc123defg \
  --content "Revised requirement" \
  --label status=approved \
  --label reviewed-by=alice \
  --label draft=
```

**Label removal behavior:**

| Command | Effect |
|---------|--------|
| `--label key=value` | Sets label `key` to `value` |
| `--label key=` | Removes label `key` |
| `--label key=""` | Removes label `key` |

---

### dna remove

Remove an artifact.

```
dna remove <ID>

Arguments:
  <ID>    Artifact ID to remove
```

**Example:**

```bash
dna remove abc123defg
```

---

### dna search

Semantic search across artifacts.

```
dna search <QUERY> [OPTIONS]

Arguments:
  <QUERY>   Natural language search query.
            Searches both content and context embeddings.

Options:
      --kind <KIND>
              Filter to specific kind.

  -l, --label <KEY=VALUE>
              Filter by label. Can be repeated for AND logic.

      --limit <N>
              Maximum results to return. [default: 10]

      --no-context
              Search content embeddings only, ignore context.

      --context-weight <WEIGHT>
              Blend weight for context similarity. [default: 0.3]
              0.0 = content only, 1.0 = context only

  -h, --help  Print help
```

**Examples:**

```bash
# Basic search
dna search "user authentication"

# Search within a kind
dna search "password reset" --kind intent

# Search with label filter
dna search "API validation" --label domain=auth

# Combine filters
dna search "performance requirements" --kind evaluation --label priority=high --limit 5
```

---

### dna list

List artifacts with optional filters.

```
dna list [OPTIONS]

Options:
      --kind <KIND>
              Filter by kind.

  -l, --label <KEY=VALUE>
              Filter by label. Can be repeated.

      --after <DATE>
              Only artifacts created after this date.
              Format: YYYY-MM-DD or ISO 8601

      --before <DATE>
              Only artifacts created before this date.

      --limit <N>
              Maximum results. [default: 100]

  -h, --help  Print help
```

**Examples:**

```bash
# List all
dna list

# List by kind
dna list --kind intent

# List with label filter
dna list --label domain=auth --label status=approved

# Recent artifacts
dna list --after 2024-01-01 --limit 20
```

---

### dna reindex

Rebuild embeddings for existing artifacts.

```
dna reindex [OPTIONS]

What to reindex:
      --all        Reindex all embeddings (content + context)
      --content    Reindex content embeddings only
      --context    Reindex context embeddings only

Filter which artifacts:
      --kind <KIND>
              Only artifacts of this kind

  -l, --label <KEY=VALUE>
              Only artifacts matching label(s)

      --id <ID>
              Specific artifact by ID

      --since <DATE>
              Only artifacts modified after date

Options:
      --dry-run    Show what would be reindexed without doing it
      --force      Reindex even if model hasn't changed

  -h, --help       Print help
```

Use `reindex` when:
- Embedding model changes
- Context is added to existing artifacts via migration
- Troubleshooting search quality issues

**Examples:**

```bash
# Reindex everything
dna reindex --all

# Reindex only context embeddings (after adding context to old artifacts)
dna reindex --context

# Reindex specific kind
dna reindex --all --kind intent

# Dry run to see what would change
dna reindex --all --dry-run
```

---

## Configuration

Configuration is stored in `.dna/config.toml`.

```toml
# Embedding model configuration
[model]
provider = "local"                    # "local", "openai", "ollama"
name = "BAAI/bge-small-en-v1.5"       # Model identifier

# For OpenAI
# provider = "openai"
# name = "text-embedding-3-small"
# api_key = "${OPENAI_API_KEY}"       # Environment variable reference

# Optional: Override token limit for custom models
# max_tokens = 8000

# Storage configuration
[storage]
# uri = "s3://bucket/path"            # Remote storage (optional)

# Registered kinds
[kinds]
definitions = [
  { slug = "intent", description = "High-level user goals..." },
  { slug = "contract", description = "API contracts..." },
]
```

### Model Token Limits

DNA includes a registry of known model token limits:

| Model | Provider | Max Tokens | Dimensions |
|-------|----------|------------|------------|
| BAAI/bge-small-en-v1.5 | local | 512 | 384 |
| BAAI/bge-base-en-v1.5 | local | 512 | 768 |
| text-embedding-3-small | openai | 8,191 | 1,536 |
| text-embedding-3-large | openai | 8,191 | 3,072 |
| nomic-embed-text | ollama | 8,192 | 768 |

For models not in the registry, DNA defaults to a conservative 512 token limit. Override with:

```toml
[model]
provider = "custom"
name = "my-custom-model"
max_tokens = 4096    # Your model's actual limit
```

### Overflow Behavior

When content or context exceeds the model's token limit, DNA returns an error:

```
Error: Content exceeds maximum of 512 tokens (got 847).
Reduce content length or configure a model with a larger context window.
```

This is intentional. Truncated embeddings lose semantic information and degrade search quality. If your content is consistently long, use a model with a larger context window.

---

## Labels vs Context

Both labels and context help with retrieval, but serve different purposes:

| Feature | Labels | Context |
|---------|--------|---------|
| Format | Structured key=value | Free-form text |
| Search | Exact match filter | Semantic similarity |
| Use case | Categorization, filtering | Relationships, domain knowledge |
| Example | `domain=auth` | "Part of the auth system, relates to GDPR" |

**Use labels when:**
- You need exact filtering (`--label domain=auth`)
- Values are enumerable (status, priority, owner)
- Categorizing for organization

**Use context when:**
- Related concepts should surface this artifact
- The content doesn't mention relevant terms directly
- Describing relationships to other artifacts or systems
- Adding domain knowledge that aids discovery

**Best practice:** Use both together.

```bash
dna add intent "User can export their data in CSV format" \
  --label domain=users \
  --label domain=gdpr \
  --label priority=high \
  --context "GDPR Article 20 right to data portability. Related to user-data-service and export-worker. Must complete within 72 hours of request."
```

---

## Error Reference

| Error | Cause | Solution |
|-------|-------|----------|
| `DNA not initialized` | No `.dna/` directory | Run `dna init` |
| `Kind not registered` | Using unregistered kind | Run `dna kind add <name> <description>` |
| `Kind slug is reserved` | Using reserved word | Choose different name |
| `Content exceeds maximum tokens` | Content too long for model | Shorten content or use larger model |
| `Context exceeds maximum tokens` | Context too long for model | Shorten context or use larger model |
| `Artifact not found` | Invalid ID | Check ID with `dna list` |

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error (invalid input, not found, etc) |
| 2 | Configuration error |

---

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `OPENAI_API_KEY` | API key for OpenAI embeddings |
| `DNA_LOG` | Log level (error, warn, info, debug, trace) |
