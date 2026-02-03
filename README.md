<div align="center">

# DNA

<img src=".github/logo.jpg" alt="DNA - Truth Artifact Management CLI" width="100%">

[![GitHub Release](https://img.shields.io/github/v/release/panbanda/dna)](https://github.com/panbanda/dna/releases/latest)
[![CI Status](https://github.com/panbanda/dna/workflows/CI/badge.svg)](https://github.com/panbanda/dna/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**The genetic code of your software.**

Capture the essential identity of your system - decisions, constraints, and guarantees that define what it *is*, independent of implementation. Truth artifacts persist while frameworks evolve, teams change, and code gets rewritten.

[Documentation](https://panbanda.github.io/dna) | [Installation](#installation) | [Quick Start](#quick-start)

</div>

---

## Installation

```bash
# Homebrew (macOS/Linux)
brew install panbanda/dna/dna

# Or download binary from releases
curl -LO https://github.com/panbanda/dna/releases/latest/download/dna-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m).tar.gz
tar xzf dna-*.tar.gz && sudo mv dna /usr/local/bin/
```

## Quick Start

```bash
# Initialize with a template
dna init --template intent

# Add truth artifacts
dna add intent "Orders must not ship until payment confirmed"
dna add contract "POST /orders returns 201 with order_id"
dna add constraint "API response time < 500ms at p95"

# Search by meaning
dna search "payment requirements"

# List artifacts
dna list --kind intent
```

## Templates

DNA provides predefined templates for common use cases:

| Template | Use Case | Kinds |
|----------|----------|-------|
| `intent` | Software systems | 11 kinds: intent, contract, algorithm, evaluation, pace, monitor, glossary, integration, reporting, compliance, constraint |
| `agentic` | AI agents | 5 kinds: behavior, boundary, threat, eval, governance |

### Intent Template

Truth artifacts that constrain implementations across any language or framework:

```mermaid
flowchart LR
    subgraph discovery["Discovery Agents"]
        architect["Architect"]
        analyst["Analyst"]
        auditor["Compliance"]
    end

    subgraph dna["DNA Layer"]
        intent(["intent"])
        contract(["contract"])
        algorithm(["algorithm"])
        evaluation(["evaluation"])
        constraint(["constraint"])
        compliance(["compliance"])
    end

    subgraph coding["Coding Agents"]
        coder["Coder"]
        tester["Tester"]
        reviewer["Reviewer"]
    end

    subgraph impl["Implementations"]
        go(["Go"])
        py(["Python"])
        any(["any..."])
    end

    architect -.->|discovers| intent
    analyst -.->|extracts| contract
    auditor -.->|updates| compliance

    intent -->|guides| coder
    contract -->|guides| coder
    algorithm -->|guides| coder
    evaluation -->|validates| tester
    constraint -->|bounds| tester
    constraint -->|bounds| reviewer
    compliance -->|enforces| reviewer

    coder ==>|produces| go
    coder ==>|produces| py
    coder ==>|produces| any

    style architect fill:#10b981,stroke:#059669,color:#fff
    style analyst fill:#10b981,stroke:#059669,color:#fff
    style auditor fill:#10b981,stroke:#059669,color:#fff

    style intent fill:#fbbf24,stroke:#d97706,color:#000
    style contract fill:#fbbf24,stroke:#d97706,color:#000
    style algorithm fill:#fbbf24,stroke:#d97706,color:#000
    style evaluation fill:#fbbf24,stroke:#d97706,color:#000
    style constraint fill:#f87171,stroke:#dc2626,color:#000
    style compliance fill:#f87171,stroke:#dc2626,color:#000

    style coder fill:#8b5cf6,stroke:#7c3aed,color:#fff
    style tester fill:#8b5cf6,stroke:#7c3aed,color:#fff
    style reviewer fill:#8b5cf6,stroke:#7c3aed,color:#fff

    style go fill:#6b7280,stroke:#4b5563,color:#fff
    style py fill:#6b7280,stroke:#4b5563,color:#fff
    style any fill:#6b7280,stroke:#4b5563,color:#fff
```

### Agentic Template

Truth artifacts for AI agent safety and governance:

```mermaid
flowchart LR
    subgraph discovery["Governance Agents"]
        hunter["Threat Hunter"]
        policy["Policy"]
        eval_agent["Eval"]
    end

    subgraph dna["DNA Layer"]
        behavior(["behavior"])
        boundary(["boundary"])
        threat(["threat"])
        eval(["eval"])
        governance(["governance"])
    end

    subgraph coding["Implementation Agents"]
        safety["Safety Coder"]
        redteam["Red Team"]
        monitor["Monitor"]
    end

    subgraph impl["AI Systems"]
        llm(["LLM Agent"])
        rag(["RAG"])
        any(["any..."])
    end

    hunter -.->|discovers| threat
    policy -.->|defines| boundary
    policy -.->|defines| governance
    eval_agent -.->|creates| eval

    behavior -->|shapes| safety
    boundary -->|limits| safety
    boundary -->|limits| monitor
    threat -->|targets| redteam
    eval -->|validates| redteam
    governance -->|enforces| monitor

    safety ==>|produces| llm
    safety ==>|produces| rag
    safety ==>|produces| any

    style hunter fill:#db2777,stroke:#be185d,color:#fff
    style policy fill:#db2777,stroke:#be185d,color:#fff
    style eval_agent fill:#db2777,stroke:#be185d,color:#fff

    style behavior fill:#60a5fa,stroke:#2563eb,color:#000
    style boundary fill:#f87171,stroke:#dc2626,color:#000
    style threat fill:#f87171,stroke:#dc2626,color:#000
    style eval fill:#60a5fa,stroke:#2563eb,color:#000
    style governance fill:#a78bfa,stroke:#7c3aed,color:#000

    style safety fill:#8b5cf6,stroke:#7c3aed,color:#fff
    style redteam fill:#8b5cf6,stroke:#7c3aed,color:#fff
    style monitor fill:#8b5cf6,stroke:#7c3aed,color:#fff

    style llm fill:#6b7280,stroke:#4b5563,color:#fff
    style rag fill:#6b7280,stroke:#4b5563,color:#fff
    style any fill:#6b7280,stroke:#4b5563,color:#fff
```

```bash
# List available templates
dna init --list-templates

# Initialize with AI safety template
dna init --template agentic
```

See [docs/templates/](docs/templates/) for detailed documentation on each template.

## MCP Integration

DNA works with AI agents via Model Context Protocol:

```json
{
  "mcpServers": {
    "dna": {
      "command": "dna",
      "args": ["mcp", "stdio"]
    }
  }
}
```

## Configuration

Configure embedding providers in `.dna/config.toml`:

```toml
# Local (default, no setup required)
[model]
provider = "local"
name = "BAAI/bge-small-en-v1.5"

# OpenAI
[model]
provider = "openai"
name = "text-embedding-3-small"
api_key = "sk-..."

# Ollama
[model]
provider = "ollama"
name = "nomic-embed-text"
base_url = "http://localhost:11434"
```

## Development

```bash
git clone https://github.com/panbanda/dna.git
cd dna
cargo build --release
cargo test
```

## License

MIT - see [LICENSE](LICENSE)

## Links

- [Full Documentation](https://panbanda.github.io/dna)
- [Template Reference](docs/templates/)
- [GitHub Issues](https://github.com/panbanda/dna/issues)
- [Changelog](CHANGELOG.md)

---

*Methodology inspired by [Intent-Starter](https://github.com/billeisenhauer/intent-starter) by Bill Eisenhauer*
