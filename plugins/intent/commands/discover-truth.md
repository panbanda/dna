# Discover Truth

Analyze a codebase and extract truth artifacts into DNA, discriminating between intentional system identity and technical debt.

## Prerequisites

1. DNA must be initialized in the target project: `dna init --template intent`
2. The project must have source code and ideally git history.

## Workflow

### Phase 1: Reconnaissance

Before spawning discovery agents, map the project and available data sources.

#### 1a. Map the codebase

Detect languages, frameworks, and structure:

```
Glob: **/*.rs, **/*.go, **/*.py, **/*.ts, **/*.js, **/*.java, **/*.rb
Glob: **/Cargo.toml, **/package.json, **/go.mod, **/requirements.txt, **/Gemfile
```

Map project structure:
- Source code directories
- Test directories
- Documentation directories
- Configuration files
- CI/CD configuration
- API specs (OpenAPI, GraphQL schemas, protobuf)

Find entry points:

```
Glob: **/main.rs, **/main.go, **/app.ts, **/index.ts
Glob: **/routes/**, **/handlers/**, **/controllers/**
```

Find documentation:

```
Glob: **/README.md, **/CLAUDE.md, **/CONTRIBUTING.md
Glob: **/docs/**/*.md, **/adr/**/*.md, **/decisions/**/*.md
```

#### 1b. Detect external data sources

Ask the user which external sources are available. Probe for MCP tools:

- **Issue tracker** (Linear, Jira, GitHub Issues): Tickets reveal intents (why was this built?), acceptance criteria (evaluations), and known debt.
- **Documentation** (Confluence, Notion, Google Docs): Architecture docs, design docs, runbooks contain intents and contracts.
- **Pull requests** (GitHub, GitLab): PR descriptions explain WHY changes were made. Review comments reveal constraints.
- **Conversations** (Slack, Teams): Decision threads capture intents that never made it to code or docs. High noise.
- **Monitoring** (Datadog, Grafana, PagerDuty): Alert configs reveal monitors and constraints.

For each available source, note the MCP tool name and access scope. Discovery agents will use these when present.

Not all sources need to be available. Code-only discovery works, but external sources add significant depth, especially for intents (the WHY behind decisions).

#### 1c. Write the recon summary

Create staging directory and write to `.dna/discovery/recon.json`:

```json
{
  "languages": ["rust", "typescript"],
  "frameworks": ["axum", "react"],
  "source_dirs": ["src/", "crates/"],
  "test_dirs": ["tests/", "e2e/"],
  "doc_files": ["README.md", "docs/architecture.md"],
  "api_specs": ["openapi.yaml"],
  "entry_points": ["src/main.rs", "src/routes/mod.rs"],
  "ci_config": [".github/workflows/ci.yml"],
  "config_files": ["config.toml", ".env.example"],
  "external_sources": {
    "issues": {"available": true, "tool": "mcp__linear__list_issues", "scope": "project X"},
    "docs": {"available": true, "tool": "mcp__notion__search", "scope": "engineering workspace"},
    "prs": {"available": true, "tool": "gh pr list", "scope": "this repo"},
    "conversations": {"available": false},
    "monitoring": {"available": false}
  }
}
```

### Phase 2: Discovery (Spawn All Agents in Parallel)

Use the Task tool to spawn all 11 agents simultaneously. Each agent reads the recon summary, explores the codebase, and optionally queries external sources to find candidates for its assigned kind.

Each agent writes its output to `.dna/discovery/<kind>.json` with this schema:

```json
{
  "kind": "intent",
  "candidates": [
    {
      "name": "Short descriptive name",
      "content": "The artifact content, written to survive a rewrite",
      "format": "markdown",
      "labels": {"area": "billing"},
      "context": "Additional semantic context for search",
      "source": {
        "type": "code|doc|ticket|pr|commit|config|test|conversation",
        "location": "file:line, URL, or ticket ID",
        "excerpt": "The relevant snippet that led to this candidate"
      },
      "classification": "truth|debt|ambiguous",
      "confidence": "high|medium|low",
      "reasoning": "Why this classification and this kind"
    }
  ]
}
```

**Classification rules for all agents:**

- **truth**: The pattern is intentional, consistent across the codebase, and supported by documentation, tests, or explicit design decisions. Storing it preserves system identity.
- **debt**: The pattern exists but should not be preserved. Signals: TODO/HACK/FIXME comments, contradicts other patterns in the codebase, is a known anti-pattern, has open tickets to change it, or would embarrass the original author.
- **ambiguous**: Could be intentional or accidental. No supporting evidence either way. Needs human judgment.

---

**Use the intent-discoverer agent** to analyze the codebase and write `.dna/discovery/intent.json`.
Give it the contents of `.dna/discovery/recon.json`.

---

**Use the contract-discoverer agent** to analyze the codebase and write `.dna/discovery/contract.json`.
Give it the contents of `.dna/discovery/recon.json`.

---

**Use the constraint-discoverer agent** to analyze the codebase and write `.dna/discovery/constraint.json`.
Give it the contents of `.dna/discovery/recon.json`.

---

**Use the algorithm-discoverer agent** to analyze the codebase and write `.dna/discovery/algorithm.json`.
Give it the contents of `.dna/discovery/recon.json`.

---

**Use the evaluation-discoverer agent** to analyze the codebase and write `.dna/discovery/evaluation.json`.
Give it the contents of `.dna/discovery/recon.json`.

---

**Use the integration-discoverer agent** to analyze the codebase and write `.dna/discovery/integration.json`.
Give it the contents of `.dna/discovery/recon.json`.

---

**Use the glossary-discoverer agent** to analyze the codebase and write `.dna/discovery/glossary.json`.
Give it the contents of `.dna/discovery/recon.json`.

---

**Use the monitor-discoverer agent** to analyze the codebase and write `.dna/discovery/monitor.json`.
Give it the contents of `.dna/discovery/recon.json`.

---

**Use the compliance-discoverer agent** to analyze the codebase and write `.dna/discovery/compliance.json`.
Give it the contents of `.dna/discovery/recon.json`.

---

**Use the pace-discoverer agent** to analyze the codebase and write `.dna/discovery/pace.json`.
Give it the contents of `.dna/discovery/recon.json`.

---

**Use the reporting-discoverer agent** to analyze the codebase and write `.dna/discovery/reporting.json`.
Give it the contents of `.dna/discovery/recon.json`.

---

### Phase 3: Truth Judgment (After All Discovery Agents Complete)

Wait for all 11 agents to finish, then:

**Use the truth-judge agent** to read all `.dna/discovery/*.json` files (except `recon.json`).

The truth judge:

1. Reviews every candidate's classification and reasoning
2. Applies cross-cutting debt signals (open tickets about the area, contradictions between candidates from different agents, TODO/HACK patterns near the source)
3. Reclassifies where warranted -- an agent may have marked something as truth that another agent's findings reveal as debt
4. Deduplicates candidates that describe the same thing from different angles
5. Connects related artifacts across kinds (an intent that drives a constraint)
6. Promotes high-quality `ambiguous` candidates to `truth` or `debt` where cross-referencing provides clarity
7. Writes three output files:
   - `.dna/discovery/store.json` -- validated truth artifacts to store
   - `.dna/discovery/debt.json` -- technical debt findings (NOT stored as truth)
   - `.dna/discovery/review.json` -- ambiguous candidates requiring human judgment

### Phase 4: Store

Store all validated truth artifacts:

```bash
# For each entry in store.json:
dna add <kind> "<content>" \
  --name "<name>" \
  --label area=<area> \
  --label discovered=true \
  --label confidence=<confidence> \
  --context "<context>"
```

### Phase 5: Summary

Display three sections:

```
TRUTH ARTIFACTS STORED:
  Intents: N    Contracts: N    Constraints: N
  Algorithms: N    Evaluations: N    Integrations: N
  Glossary: N    Monitors: N    Compliance: N
  Pace: N    Reporting: N
  Total: N

TECHNICAL DEBT IDENTIFIED:
  (list each debt finding with source location and brief description)
  Full report: .dna/discovery/debt.json

NEEDS HUMAN REVIEW:
  (list each ambiguous candidate with the question that needs answering)
  Full report: .dna/discovery/review.json
```

Remind the user:
- Review artifacts labeled `discovered=true` for accuracy
- The debt report is actionable -- consider creating tickets
- Ambiguous candidates need a human decision: truth or debt?
