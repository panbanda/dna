# DNA CLI Design Document

**Date:** 2025-01-27
**Status:** Draft

## Overview

DNA is a Rust CLI tool for managing "truth artifacts" - the authoritative statements that define what a system is and must remain, independent of implementation.

### Goals (in priority order)

1. **Agent memory/context** - Agents query DNA to understand system identity before making changes
2. **Drift detection** - Surface truth artifacts relevant to code changes for comparison
3. **Change impact analysis** - Track what truth changed over time

### Key Properties

- Embedded LanceDB for vector storage (no external database)
- Local-first embedding with optional remote providers
- Project-scoped configuration (no global config)
- MCP server for agent integration
- Cross-platform (Windows, Linux, macOS)
- Rust 1.84

### Non-goals (initial scope)

- File import/round-tripping
- Author tracking (just timestamps)
- Built-in drift analysis logic (agents do the reasoning)

---

## Data Model

### Artifact Types

| Type | Description |
|------|-------------|
| `intent` | What the system is/does (domain definitions) |
| `invariant` | What must always be true |
| `contract` | External API promises |
| `algorithm` | Computation rules/domain logic |
| `evaluation` | Test scenarios/acceptance criteria |
| `pace` | Change permission policies |
| `monitor` | Observable contracts (SLOs, metrics) |

### Artifact Schema

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | 10-char lowercase alphanumeric (nanoid) |
| `type` | enum | One of the 7 artifact types |
| `name` | string (optional) | Slugified short name for filename |
| `content` | string | The artifact body (markdown, YAML, etc.) |
| `format` | enum | `markdown`, `yaml`, `json`, `openapi`, `text` |
| `metadata` | JSON | Flexible key-value pairs (domain, service, tags, etc.) |
| `embedding` | vector | The embedded content |
| `embedding_model` | string | Which model produced the embedding |
| `created_at` | timestamp | When created |
| `updated_at` | timestamp | When last modified |

### ID Format

10-character lowercase alphanumeric using a reduced alphabet (no ambiguous characters):

```rust
const ALPHABET: &str = "23456789abcdefghjkmnpqrstuvwxyz";
// 10 chars, ~49 bits entropy, 1-2 LLM tokens
// Example: "k7v3m9xnp2"
```

---

## Directory Structure

### Project structure

```
.dna/
  config.toml           # project config
  db/
    artifacts.lance/    # LanceDB vector table

~/.dna/
  models/               # cached local models (shared across projects)
    BAAI--bge-small-en-v1.5/
    ...
```

### Configuration

Project config lives in `.dna/config.toml`:

```toml
[model]
provider = "local"                        # local | openai | ollama
name = "BAAI/bge-small-en-v1.5"

# OpenAI example
# [model]
# provider = "openai"
# name = "text-embedding-3-small"
# api_key_env = "OPENAI_API_KEY"

# Ollama example
# [model]
# provider = "ollama"
# name = "nomic-embed-text"
# base_url = "http://localhost:11434"     # optional, this is default
```

### Render Output

Default output directory: `./dna/`

```
dna/
  checkout/
    invariants/
      valid-payment.md
      unique-email-addresses.md
    contracts/
      ...
  inventory/
    ...
```

### Rendered File Format

Frontmatter + content:

```markdown
---
id: k7v3m9xnp2
type: invariant
format: markdown
metadata:
  domain: checkout
  priority: high
created_at: 2024-01-15T10:30:00Z
updated_at: 2024-01-15T10:30:00Z
---

Users must have a valid payment method before completing checkout.
```

### Filename Resolution

1. Use `name` if provided (slugified via `slug` crate)
2. Otherwise, slugify first ~50 chars of content
3. On conflict, append 8-char ID suffix: `valid-payment-k7v3m9xn.md`
4. Last resort, use full ID: `k7v3m9xnp2.md`

---

## CLI Commands

```bash
# Initialization
dna init [--model <provider:model>]

# Artifact CRUD (for each type: intent, invariant, contract, algorithm, evaluation, pace, monitor)
dna <type> add "<content>" [--name <slug>] [--format <fmt>] [--meta key=val]...
dna <type> get <id>
dna <type> update <id> [--content "..."] [--name ...] [--meta key=val]...
dna <type> remove <id>
dna <type> list [--filter key=val]... [--since <timestamp>] [--limit N]

# Cross-cutting
dna search "<query>" [--type <type>] [--filter key=val]... [--limit N]
dna list [--type <type>] [--filter key=val]... [--since <timestamp>]
dna changes [--since <timestamp|git-ref>]
dna render [--by <key>[,<key>]] [--output <dir>]
dna reindex [--force]

# Configuration
dna config model [<provider:model>]
dna config get <key>
dna config set <key> <value>

# MCP Server
dna mcp [--include <tool>,...]  [--exclude <tool>,...]
```

---

## Embedding Providers

### Supported Providers

| Provider | Description |
|----------|-------------|
| `local` | HuggingFace models via Candle (default) |
| `openai` | OpenAI API |
| `ollama` | Local Ollama service |

**Default model:** `BAAI/bge-small-en-v1.5` (local)

### Provider Trait

```rust
pub trait EmbeddingProvider {
    fn embed(&self, text: &str) -> Result<Vec<f32>>;
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
    fn model_id(&self) -> &str;
    fn dimensions(&self) -> usize;
}
```

### Reindexing

- `embedding_model` stored with each artifact
- `dna config model <new-model>` prompts to reindex if model changes
- `dna reindex` re-embeds all artifacts with current model
- Queries warn if embeddings are stale (mixed models)

---

## MCP Server

### Command

```bash
dna mcp [--include <tools>] [--exclude <tools>]
```

### Available Tools

| Tool | Type | Description |
|------|------|-------------|
| `dna_search` | read | Semantic search with filters |
| `dna_get` | read | Get artifact by ID |
| `dna_list` | read | List artifacts by type/metadata |
| `dna_changes` | read | Artifacts modified since timestamp/git-ref |
| `dna_add` | write | Add new artifact |
| `dna_update` | write | Modify existing artifact |
| `dna_remove` | write | Delete artifact |

### Filtering Examples

```bash
dna mcp                                  # all tools
dna mcp --exclude add,update,remove      # read-only
dna mcp --include search,get,changes     # minimal context-fetching
```

### Tool Schema Example

```json
{
  "name": "dna_search",
  "description": "Semantic search for truth artifacts",
  "parameters": {
    "query": { "type": "string", "required": true },
    "type": { "type": "string", "enum": ["intent", "invariant", "contract", "algorithm", "evaluation", "pace", "monitor"] },
    "filter": { "type": "object", "description": "key=value metadata filters" },
    "limit": { "type": "integer", "default": 10 }
  }
}
```

---

## Dependencies & Architecture

### Key Rust Crates

| Crate | Purpose |
|-------|---------|
| `lancedb` | Embedded vector database |
| `candle-core`, `candle-nn`, `candle-transformers` | Local model inference |
| `hf-hub` | HuggingFace model downloads |
| `clap` | CLI argument parsing |
| `serde`, `serde_json`, `serde_yaml` | Serialization |
| `toml` | Config file parsing |
| `nanoid` | ID generation |
| `slug` | Filename slugification |
| `chrono` | Timestamps |
| `reqwest` | HTTP for OpenAI/Ollama APIs |
| `tokio` | Async runtime |
| `dirs` | Cross-platform path resolution |

### Module Structure

```
src/
  main.rs
  cli/
    mod.rs
    init.rs
    artifact.rs      # CRUD for all artifact types
    search.rs
    render.rs
    config.rs
    mcp.rs
  db/
    mod.rs
    schema.rs        # artifact struct, queries
    lance.rs         # LanceDB operations
  embedding/
    mod.rs
    provider.rs      # trait definition
    local.rs         # Candle implementation
    openai.rs
    ollama.rs
  render/
    mod.rs           # render to files
```

### Cross-platform Considerations

- Use `dirs` crate for `~/.dna/` path resolution
- Avoid Unix-specific APIs
- Test CI on Windows, Linux, macOS

---

## Summary

A Rust CLI (`dna`) for managing truth artifacts with vector search, designed for agent-driven development workflows.

**Core features:**
- 7 artifact types: intent, invariant, contract, algorithm, evaluation, pace, monitor
- Flat storage with rich metadata, structure emerges at render time
- Embedded LanceDB, project-scoped (`.dna/`)
- Local-first embeddings (Candle) with OpenAI/Ollama options
- MCP server for agent integration with include/exclude tool filtering
- Cross-platform (Windows, Linux, macOS), Rust 1.84

**Deferred:**
- File import/round-tripping
- Author tracking
- Built-in drift analysis (agents do the reasoning)
