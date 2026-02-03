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
flowchart RL
    subgraph impl["Implementations"]
        go["Go App"]
        py["Python App"]
        any["...any language"]
    end

    subgraph coding["Coding Agents"]
        coder["Coder<br/><sub>intent, contract, algorithm</sub>"]
        tester["Tester<br/><sub>evaluation, constraint</sub>"]
        reviewer["Reviewer<br/><sub>compliance, constraint</sub>"]
    end

    subgraph dna["DNA Layer"]
        intent["intent"]
        contract["contract"]
        algorithm["algorithm"]
        evaluation["evaluation"]
        constraint["constraint"]
        compliance["compliance"]
        more["..."]
    end

    subgraph discovery["Discovery Agents"]
        architect["Architect<br/><sub>discovers intent</sub>"]
        analyst["Analyst<br/><sub>extracts contracts</sub>"]
        auditor["Compliance<br/><sub>updates regulations</sub>"]
    end

    architect --> intent
    analyst --> contract
    auditor --> compliance
    intent --> coder
    contract --> coder
    algorithm --> coder
    evaluation --> tester
    constraint --> tester
    constraint --> reviewer
    compliance --> reviewer
    coder --> go
    coder --> py
    coder --> any

    style discovery fill:#d1fae5,stroke:#059669
    style dna fill:#fef3c7,stroke:#d97706
    style coding fill:#ede9fe,stroke:#7c3aed
    style impl fill:#f3f4f6,stroke:#6b7280
```

### Agentic Template

Truth artifacts for AI agent safety and governance:

```mermaid
flowchart RL
    subgraph impl["AI Systems"]
        llm["LLM Agent"]
        rag["RAG Pipeline"]
        any["...any AI system"]
    end

    subgraph coding["Implementation Agents"]
        safety["Safety Coder<br/><sub>behavior, boundary</sub>"]
        redteam["Red Team<br/><sub>threat, eval</sub>"]
        monitor["Monitor<br/><sub>governance, boundary</sub>"]
    end

    subgraph dna["DNA Layer"]
        behavior["behavior"]
        boundary["boundary"]
        threat["threat"]
        eval["eval"]
        governance["governance"]
    end

    subgraph discovery["Governance Agents"]
        hunter["Threat Hunter<br/><sub>discovers risks</sub>"]
        policy["Policy<br/><sub>defines boundaries</sub>"]
        eval_agent["Eval<br/><sub>creates benchmarks</sub>"]
    end

    hunter --> threat
    policy --> boundary
    policy --> governance
    eval_agent --> eval
    behavior --> safety
    boundary --> safety
    boundary --> monitor
    threat --> redteam
    eval --> redteam
    governance --> monitor
    safety --> llm
    safety --> rag
    safety --> any

    style discovery fill:#fce7f3,stroke:#db2777
    style dna fill:#dbeafe,stroke:#2563eb
    style coding fill:#ede9fe,stroke:#7c3aed
    style impl fill:#f3f4f6,stroke:#6b7280
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
