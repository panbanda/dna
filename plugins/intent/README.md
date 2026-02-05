# Intent Template Commands

## discover-truth

Analyze an existing codebase and extract truth artifacts into DNA, discriminating between intentional system identity and technical debt.

### Usage

Initialize DNA in the target project, then invoke the command:

```bash
cd /path/to/target-project
dna init --template intent
```

The command is orchestrated by Claude Code. It spawns subagents to analyze the codebase in parallel.

### What it does

The command runs in 5 phases:

```
RECON ──> DISCOVER ──> JUDGE ──> STORE ──> SUMMARY
  1 agent   11 agents    1 agent   CLI       output
             parallel
```

**Phase 1 -- Reconnaissance**: A single agent maps the project structure (languages, frameworks, entry points, docs, API specs), identifies product capabilities (major user-facing features), and detects available external data sources (issue trackers, documentation tools, PR history, monitoring). Writes `.dna/discovery/recon.json`.

**Phase 2 -- Discovery**: 11 agents run in parallel, one per artifact kind. Each reads the recon summary, adapts its search patterns to the detected language and framework, explores the codebase, and optionally queries external sources via MCP tools. Each writes candidates to `.dna/discovery/<kind>.json`. All candidate content must be language-agnostic -- no framework names, no code syntax, no implementation details.

**Phase 3 -- Truth Judgment**: A single judge agent reads all candidates and applies cross-cutting analysis. Produces three output files separating truth from debt from ambiguity.

**Phase 4 -- Store**: Validated truth artifacts are stored in DNA with `discovered=true` label.

**Phase 5 -- Summary**: Displays counts of stored truth, identified debt, and items needing human review.

### Discovery agents

Each agent focuses on one artifact kind and knows where evidence hides for that kind:

| Agent | Kind | Primary sources | What it looks for |
|-------|------|-----------------|-------------------|
| intent-discoverer | intent | ADRs, code comments, git history, PR descriptions, product capabilities | WHY decisions were made. Product features and their purpose. Enterprise/tier requirements. |
| contract-discoverer | contract | API routes, OpenAPI specs, public types, event schemas | Component BOUNDARIES. What is promised vs. what happens to exist. |
| constraint-discoverer | constraint | Validation logic, middleware, config limits, error handlers | Hard LIMITS. What the system rejects reveals what it considers unacceptable. |
| algorithm-discoverer | algorithm | Business logic, pricing, scoring, domain arithmetic | Computation LOGIC. Functions with domain variable names, not generic CRUD. |
| evaluation-discoverer | evaluation | Test suites, CI config, assertion patterns, acceptance criteria | SUCCESS criteria. Test names that describe business rules, not implementation. |
| integration-discoverer | integration | SDK imports, API clients, env vars, docker-compose | External DEPENDENCIES and assumptions about them. |
| glossary-discoverer | glossary | Type definitions, enums, domain models, inconsistent terminology | Domain TERMS where meaning is specific or ambiguous. |
| monitor-discoverer | monitor | Metrics emission, alert configs, health checks, logging | OBSERVABILITY requirements and action thresholds. |
| compliance-discoverer | compliance | Data handling, consent flows, audit logs, encryption | REGULATORY obligations with source regulation cited. |
| pace-discoverer | pace | Git change frequency, CODEOWNERS, branch protection, deploy config | CHANGE VELOCITY -- which areas move fast vs. slow. |
| reporting-discoverer | reporting | Analytics events, DB views, dashboard configs, aggregation queries | BUSINESS QUERIES that must be answerable from the data. |

### External data sources

The recon phase detects available MCP tools and records them in `recon.json`. Discovery agents use these when present:

| Source | MCP tool examples | Value |
|--------|-------------------|-------|
| Issue tracker | Linear, Jira, GitHub Issues | Tickets reveal intents (why was this built?), acceptance criteria (evaluations), and known debt (open bugs/improvements). |
| Documentation | Confluence, Notion, Google Docs | Architecture docs and design docs contain intents and contracts that never made it into code. |
| Pull requests | GitHub, GitLab | PR descriptions explain WHY changes were made. Review comments reveal constraints and contested decisions. |
| Conversations | Slack, Teams | Decision threads capture intents. High noise -- use sparingly. |
| Monitoring | Datadog, Grafana, PagerDuty | Alert configs and dashboards reveal monitor artifacts and operational constraints. |

Not all sources need to be available. Code-only discovery works. External sources add depth, especially for intents (the WHY that lives outside the code).

### Truth vs. debt discrimination

The central problem: analyzing a codebase reveals patterns that ARE true but SHOULD NOT be preserved. MD5 password hashing exists in the code, but storing it as a constraint canonizes a vulnerability. A `sleep(5)` retry exists, but it is not an algorithm to preserve.

Three layers prevent canonizing debt:

**Layer 1 -- Agent-level classification**: Each discovery agent classifies its candidates as `truth`, `debt`, or `ambiguous` using kind-specific signals. The constraint-discoverer knows MD5 is debt. The algorithm-discoverer knows `// TODO: fix this` near a formula means debt.

**Layer 2 -- Cross-referencing**: The truth judge compares candidates across agents. If the intent-discoverer finds "we use event sourcing" but the constraint-discoverer found "TODO: migrate away from event sourcing," the contradiction surfaces as ambiguous.

**Layer 3 -- Three-file output**: Results are separated into:

- `store.json` -- Validated truth. Stored in DNA.
- `debt.json` -- Technical debt findings. NOT stored. Actionable report for creating tickets.
- `review.json` -- Ambiguous candidates. Each has a specific yes/no question for a human to answer.

### Candidate schema

Every discovery agent writes candidates in this format:

```json
{
  "kind": "intent",
  "candidates": [
    {
      "name": "Short descriptive name",
      "content": "The truth itself -- language-agnostic, survives a rewrite",
      "format": "markdown",
      "labels": {"area": "billing"},
      "context": "Domain concepts, related areas, brief provenance -- embedded for graph-like search",
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

The `content` field is the truth itself -- language-agnostic, no implementation details. The `context` field is semantic glue: domain concepts, related areas, and brief provenance that get embedded alongside content. Related artifacts with overlapping context cluster together in search, creating graph-like connections across kinds. The `source` field traces provenance to specific code, docs, or tickets. The `classification` and `reasoning` fields are the agent's judgment, subject to override by the truth judge.

### Classification criteria

**truth** -- The pattern is intentional. Signals:
- Documented in ADRs, comments, or tickets
- Verified by tests
- Consistent across the entire codebase
- Supported by multiple independent sources (code + docs + tests)

**debt** -- The pattern exists but should not be preserved. Signals:
- TODO/HACK/FIXME/WORKAROUND/TEMPORARY comments near the source
- Open tickets to change the behavior
- Contradicts other patterns in the codebase
- Known anti-pattern (N+1 queries, god classes, weak crypto)
- Explicitly marked as deprecated

**ambiguous** -- Insufficient evidence either way. Signals:
- Code enforces a pattern but no documentation explains why
- A config value matches a framework default (intentional or accidental?)
- Pattern is partially followed (some modules comply, others don't)
- The reasoning is plausible but unverifiable from available sources

### Output

After completion, artifacts are stored in DNA with the label `discovered=true`. The summary shows:

```
TRUTH ARTIFACTS STORED:
  Intents: 8    Contracts: 5    Constraints: 12
  Algorithms: 3    Evaluations: 7    Integrations: 4
  Glossary: 6    Monitors: 3    Compliance: 2
  Pace: 3    Reporting: 1
  Total: 54

TECHNICAL DEBT IDENTIFIED:
  - [high] MD5 password hashing (auth/hash.rs:42)
  - [medium] Hardcoded retry delay of 5s (api/client.rs:118)
  - [low] Inconsistent error response format across 3 endpoints
  Full report: .dna/discovery/debt.json

NEEDS HUMAN REVIEW:
  - Is the API rate limit of 100/min intentional or a framework default?
  - Is the 30-day data retention enforced or just documented?
  Full report: .dna/discovery/review.json
```

Review stored artifacts with `dna list --label discovered=true`. The debt report is actionable -- consider creating tickets for high-severity findings.

## extract-truth

Extract truth artifacts from non-code sources -- documents, prompts, conversations, specs, incident reports, or any text input.

### When to use

Use `extract-truth` when the source is not a codebase. Use `discover-truth` for codebase analysis.

| Source type | Examples |
|-------------|----------|
| Design documents | Architecture specs, RFCs, ADRs |
| Product requirements | PRDs, user stories, acceptance criteria |
| Incident reports | Post-mortems, root cause analyses |
| Meeting notes | Decision logs, architecture reviews |
| Conversations | Interview transcripts, Slack threads |
| Existing truth directories | Another project's `truth/` directory |
| Inline text | User describing their system directly |

### How it works

Unlike `discover-truth`, this command does not spawn parallel agents or run a recon phase. It reads the input, identifies artifacts across all 11 kinds, and presents candidates for review before storing.

```
READ ──> EXTRACT ──> REVIEW ──> STORE
 input    candidates   user      CLI
                       approval
```

Key differences from `discover-truth`:
- **Interactive**: Presents candidates for user approval before storing
- **Single agent**: No parallel discovery -- one pass through the input
- **Confidence from document type**: An approved ADR gets high confidence; meeting notes get medium
- **Labels with `extracted=true`** instead of `discovered=true` to distinguish the source

### Usage

```bash
# From a file
dna extract-truth --file docs/architecture.md

# From a directory of docs
dna extract-truth --file "docs/*.md"

# From inline text (interactive)
dna extract-truth
```

Review stored artifacts with `dna list --label extracted=true`.
