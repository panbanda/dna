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

    subgraph increments["Increments"]
        pr(["PR"])
    end

    discovery -.->|updates| dna

    dna -->|context / mcp| coder
    dna -->|context / mcp| tester
    dna -->|context / mcp| reviewer

    coder ==>|produces| impl
    tester ==>|validates| pr
    reviewer ==>|approves| pr

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

    style pr fill:#22c55e,stroke:#16a34a,color:#fff
```

### Agentic Template

Truth artifacts for building and evaluating AI systems:

```mermaid
flowchart LR
    subgraph discovery["Discovery Agents"]
        threat_hunter["Threat Hunter"]
        policy_writer["Policy Writer"]
        eval_designer["Eval Designer"]
    end

    subgraph dna["DNA Layer"]
        behavior(["behavior"])
        boundary(["boundary"])
        threat(["threat"])
        evaluation(["eval"])
        governance(["governance"])
    end

    subgraph building["Building Agents"]
        prompt_eng["Prompt Engineer"]
        integrator["LLM Integrator"]
        guardrails["Guardrails Dev"]
    end

    subgraph testing["Testing Agents"]
        eval_runner["Eval Runner"]
        red_team["Red Team"]
        benchmark["Benchmarker"]
    end

    subgraph outputs["Outputs"]
        agent(["AI Agent"])
        eval_report(["Eval Report"])
        safety_score(["Safety Score"])
    end

    discovery -.->|updates| dna

    dna -->|context / mcp| building
    dna -->|context / mcp| testing

    prompt_eng ==>|produces| agent
    integrator ==>|produces| agent
    guardrails ==>|produces| agent

    eval_runner ==>|produces| eval_report
    red_team ==>|produces| safety_score
    benchmark ==>|produces| eval_report

    style threat_hunter fill:#db2777,stroke:#be185d,color:#fff
    style policy_writer fill:#db2777,stroke:#be185d,color:#fff
    style eval_designer fill:#db2777,stroke:#be185d,color:#fff

    style behavior fill:#60a5fa,stroke:#2563eb,color:#000
    style boundary fill:#f87171,stroke:#dc2626,color:#000
    style threat fill:#f87171,stroke:#dc2626,color:#000
    style evaluation fill:#60a5fa,stroke:#2563eb,color:#000
    style governance fill:#a78bfa,stroke:#7c3aed,color:#000

    style prompt_eng fill:#8b5cf6,stroke:#7c3aed,color:#fff
    style integrator fill:#8b5cf6,stroke:#7c3aed,color:#fff
    style guardrails fill:#8b5cf6,stroke:#7c3aed,color:#fff

    style eval_runner fill:#f59e0b,stroke:#d97706,color:#fff
    style red_team fill:#f59e0b,stroke:#d97706,color:#fff
    style benchmark fill:#f59e0b,stroke:#d97706,color:#fff

    style agent fill:#6b7280,stroke:#4b5563,color:#fff
    style eval_report fill:#22c55e,stroke:#16a34a,color:#fff
    style safety_score fill:#22c55e,stroke:#16a34a,color:#fff
```

```bash
# List available templates
dna init --list-templates

# Initialize with AI safety template
dna init --template agentic
```

See [docs/templates/](docs/templates/) for detailed documentation on each template.

## Plugins and Skills

DNA includes plugins that teach AI agents how to work with truth artifacts. Built on the [Agent Skills](https://agentskills.io) open standard.

### Plugin structure

```
.claude-plugin/
  marketplace.json          # Plugin registry

plugins/
  core/                     # Works with any DNA template
    skills/
      planning-with-truth/  # Search DNA before choosing an approach
      validating-approach/  # Check plans against known truth
      checking-truth/       # Pre-change verification
      capturing-artifacts/  # How to write good artifacts
      searching-artifacts/  # Query patterns and score interpretation
      managing-kinds/       # Custom kind registration

  intent/                   # Intent template specific
    skills/
      writing-intents/      # How to write intent artifacts
      writing-contracts/    # How to write contract artifacts
      writing-constraints/  # How to write constraint artifacts
      writing-algorithms/   # How to write algorithm artifacts
      writing-evaluations/  # How to write evaluation artifacts
      reviewing-compliance/ # Post-implementation truth review
    commands/
      discover-truth        # Analyze a codebase with 11 parallel agents
      extract-truth         # Extract artifacts from documents and prompts
    agents/                 # Discovery agent instructions (11 + judge)
    reference/              # Kind reference documentation
```

### Commands

| Command | Source | What it does |
|---------|--------|-------------|
| `discover-truth` | Codebase | Spawns 11 parallel agents to analyze code, tests, config, and external sources. A truth judge separates truth from debt from ambiguity. |
| `extract-truth` | Documents, prompts | Single-pass extraction from design docs, PRDs, incident reports, meeting notes, or inline text. Presents candidates for user review before storing. |

### Discovery workflow

```
RECON --> DISCOVER --> JUDGE --> STORE --> SUMMARY
1 agent   11 agents   1 agent   CLI       output
           parallel
```

The recon phase maps the codebase, identifies product capabilities, and detects available external sources (issue trackers, docs, PRs via MCP). Discovery agents adapt their search patterns to the detected language and framework. All artifact output is language-agnostic -- it must survive a rewrite.

Results are separated into three files:
- `store.json` -- validated truth, stored in DNA
- `debt.json` -- technical debt findings, not stored
- `review.json` -- ambiguous candidates needing human judgment

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
