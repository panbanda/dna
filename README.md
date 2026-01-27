# DNA - Truth Artifact Management CLI

[![CI Status](https://github.com/yourusername/dna/workflows/CI/badge.svg)](https://github.com/yourusername/dna/actions)
[![Coverage](https://codecov.io/gh/yourusername/dna/branch/main/graph/badge.svg)](https://codecov.io/gh/yourusername/dna)
[![Crates.io](https://img.shields.io/crates/v/dna.svg)](https://crates.io/crates/dna)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> **Transform how AI agents understand your codebase** - Make intent explicit, searchable, and persistent.

DNA is a powerful CLI tool for managing "truth artifacts" - authoritative statements that define what your system is and must remain, independent of implementation details. Built for AI agents and human developers working together.

## Why DNA?

### The Problem
Traditional code loses context over time. Why was this decision made? What invariants must hold? How do different components relate? This knowledge lives in:
- Long-forgotten commit messages
- Stale documentation
- Tribal knowledge of departed team members
- The minds of AI agents that generated code

### The Solution: Intent-Starter Methodology

DNA implements the **Intent-Starter** approach - making intent explicit, searchable, and persistent:

```bash
# Declare an intent (the "why")
dna intent add "Authentication must support both OAuth and API keys to serve web and programmatic users"

# Define an invariant (what must always be true)
dna invariant add "User passwords must be hashed with bcrypt cost factor >= 12"

# Specify a contract (guaranteed behavior)
dna contract add "POST /api/auth returns 401 for invalid credentials within 100ms"
```

Later, when you (or an AI agent) asks "Why do we support multiple auth methods?":

```bash
dna search "authentication methods"
# Instantly finds the intent artifact with full context
```

## Key Features

- **7 Artifact Types**: intent, invariant, contract, algorithm, evaluation, pace, monitor
- **Semantic Search**: Vector-powered search finds artifacts by meaning, not keywords
- **Flexible Embedding**: Local (Candle), OpenAI, or Ollama providers
- **MCP Integration**: Native Model Context Protocol server for AI agents
- **Project-Scoped**: Each project is independent, no global configuration
- **Cross-Platform**: Linux, macOS, Windows support with native binaries

## Installation

### Pre-built Binaries (Recommended)

Download the latest release for your platform:

```bash
# Linux x86_64
curl -LO https://github.com/yourusername/dna/releases/latest/download/dna-linux-x86_64.tar.gz
tar xzf dna-linux-x86_64.tar.gz
sudo mv dna /usr/local/bin/

# macOS (Apple Silicon)
curl -LO https://github.com/yourusername/dna/releases/latest/download/dna-macos-aarch64.tar.gz
tar xzf dna-macos-aarch64.tar.gz
sudo mv dna /usr/local/bin/

# Windows (download and add to PATH)
# https://github.com/yourusername/dna/releases/latest/download/dna-windows-x86_64.zip
```

### From Source

```bash
cargo install dna
# or
git clone https://github.com/yourusername/dna.git
cd dna
cargo build --release
```

## Quick Start

### 1. Initialize Your Project

```bash
dna init
# Creates .dna/ directory with config and vector database
```

### 2. Add Artifacts

```bash
# Why does this feature exist?
dna intent add "Support real-time collaboration to enable distributed teams" --meta feature=collaboration

# What must always be true?
dna invariant add "Document versions are immutable once published" --meta domain=document-model

# What behavior is guaranteed?
dna contract add "GET /api/documents/{id} returns 404 if document doesn't exist" --meta api=documents
```

### 3. Search Semantically

```bash
# Find by meaning, not exact keywords
dna search "team working together"
# Finds: "Support real-time collaboration..."

dna search "document can't change"
# Finds: "Document versions are immutable..."
```

### 4. Organize and Render

```bash
# Group artifacts by metadata
dna render --by domain
# Creates: artifacts/domain/document-model/invariants/...

# Export as markdown
dna render --format markdown
```

## MCP Server for AI Agents

DNA includes a Model Context Protocol (MCP) server that lets AI agents interact with artifacts:

### Setup

Add to your Claude Code or other MCP-compatible client:

```json
{
  "mcpServers": {
    "dna": {
      "command": "dna",
      "args": ["mcp", "stdio"],
      "env": {
        "DNA_PROJECT_ROOT": "/path/to/your/project"
      }
    }
  }
}
```

### Available Tools

- `list_artifacts`: List all artifacts by type
- `search_artifacts`: Semantic search across all artifacts
- `get_artifact`: Retrieve specific artifact details
- `add_artifact`: Create new artifacts
- `update_artifact`: Modify existing artifacts
- `delete_artifact`: Remove artifacts

### Tool Filtering

Enable only specific tools for focused agents:

```bash
dna mcp stdio --include-tools list_artifacts,search_artifacts
# Read-only mode for safety
```

## Architecture

### Service-Oriented Design

DNA uses a clean, layered architecture:

```
┌─────────────────────────────────────┐
│     Interface Layers (Thin)         │
│  ┌──────────┐       ┌──────────┐   │
│  │   CLI    │       │   MCP    │   │
│  └──────────┘       └──────────┘   │
└─────────────────────────────────────┘
              │
┌─────────────────────────────────────┐
│    Services (Business Logic)        │
│  - artifact.rs  - search.rs         │
│  - config.rs    - types.rs          │
└─────────────────────────────────────┘
              │
┌─────────────────────────────────────┐
│      Infrastructure                 │
│  - db/ (LanceDB)                    │
│  - embedding/ (Local/OpenAI/Ollama) │
│  - render/ (File system output)     │
└─────────────────────────────────────┘
```

**Benefits**:
- Framework-agnostic core logic
- Easy to add new interfaces (REST API, gRPC)
- Fully testable in isolation
- Clear separation of concerns

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for details.

## Architecture

### Service Layer (Framework-Agnostic)
- `/src/services/` - All business logic
  - `artifact.rs` - CRUD operations
  - `search.rs` - Semantic search
  - `config.rs` - Configuration management
  - `types.rs` - Core data structures

### Interface Layers (Thin Wrappers)
- `/src/cli/` - Command-line interface
- `/src/mcp/` - MCP server protocol

### Infrastructure
- `/src/db/` - LanceDB integration
- `/src/embedding/` - Embedding providers
- `/src/render/` - File rendering

## Configuration

Configure embedding providers in `.dna/config.toml`:

```toml
[model]
provider = "local"
name = "BAAI/bge-small-en-v1.5"

# Or use OpenAI
# provider = "openai"
# name = "text-embedding-3-small"
# api_key = "sk-..."  # Or set OPENAI_API_KEY env var

# Or use Ollama
# provider = "ollama"
# name = "nomic-embed-text"
# base_url = "http://localhost:11434"
```

### Embedding Providers

| Provider | Best For | Setup |
|----------|----------|-------|
| **Local (Candle)** | Privacy, offline use | No setup required |
| **OpenAI** | Best quality, production | Requires API key |
| **Ollama** | Local + GPU acceleration | Install Ollama |

## Development

### Building from Source

```bash
git clone https://github.com/yourusername/dna.git
cd dna
cargo build --release
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with coverage
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

**Development Requirements**:
- Rust 1.84+
- Pre-commit hooks: `lefthook install`
- 95% code coverage for new code
- All clippy lints must pass

### Code Quality

DNA enforces strict quality standards:
- **Formatting**: rustfmt
- **Linting**: clippy (strict mode)
- **Testing**: 95% minimum coverage
- **Security**: cargo-audit, cargo-deny
- **Documentation**: All public APIs documented

See [docs/METHODOLOGY.md](docs/METHODOLOGY.md) for development methodology.

## Intent-Starter Methodology

DNA implements the Intent-Starter approach to software development:

1. **Make Intent Explicit**: Every feature has a documented "why"
2. **Track Invariants**: Critical properties are first-class artifacts
3. **Define Contracts**: Behavioral guarantees are explicit
4. **Enable Discovery**: Semantic search finds related decisions
5. **Maintain Context**: Knowledge persists across team changes

### Seven Artifact Types

| Type | Purpose | Example |
|------|---------|---------|
| **Intent** | Why it exists | "Support OAuth for enterprise SSO" |
| **Invariant** | What must be true | "User IDs are immutable UUIDs" |
| **Contract** | Guaranteed behavior | "API returns 200 or 4xx, never 500" |
| **Algorithm** | How it works | "Use bcrypt for password hashing" |
| **Evaluation** | Success metrics | "Search results in <100ms p95" |
| **Pace** | Timing/sequence | "Retry failed jobs with exponential backoff" |
| **Monitor** | Observability | "Log all authentication attempts" |

Read more: [docs/METHODOLOGY.md](docs/METHODOLOGY.md)

## Real-World Use Cases

### For Solo Developers
- Remember why you made decisions 6 months ago
- Onboard your future self faster
- Let AI assistants understand your project's intent

### For Teams
- Share context across team members
- Onboard new developers in hours, not weeks
- Maintain consistency across features

### For AI-Human Collaboration
- Give AI agents access to high-level intent
- Prevent AI from violating system invariants
- Enable semantic discovery of related decisions

### For Open Source
- Document design decisions for contributors
- Make project philosophy explicit
- Enable faster contribution ramp-up

## Project Status

DNA is under active development. Current status:

- [x] Core artifact management
- [x] Vector search with LanceDB
- [x] MCP server integration
- [x] Local embedding support
- [x] OpenAI/Ollama providers
- [x] File rendering
- [ ] Web interface (planned)
- [ ] API server (planned)
- [ ] Plugin system (planned)

## Performance

- **Search**: <50ms for 10k artifacts (p95)
- **Embedding**: 100-500 tokens/sec (local)
- **Storage**: ~1KB per artifact (compressed)
- **Memory**: <100MB for typical projects

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- Built with [LanceDB](https://lancedb.com/) for vector storage
- Embeddings via [Candle](https://github.com/huggingface/candle)
- Inspired by [Model Context Protocol](https://modelcontextprotocol.io/)
- Developed using [SPARC methodology](https://github.com/ruvnet/claude-flow)

## Support

- **Documentation**: [docs/](docs/)
- **Issues**: [GitHub Issues](https://github.com/yourusername/dna/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/dna/discussions)
- **Security**: See [SECURITY.md](SECURITY.md)

---

**Made with intent** by developers who believe context should never be lost.
