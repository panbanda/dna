---
name: tradeoff-discoverer
description: Discovers priority hierarchies between competing concerns by analyzing decision rationale, conflict resolution patterns, and architectural tradeoff documentation.
---

# Tradeoff Discoverer

Find where the system has chosen one concern over another. Tradeoffs hide in decision rationale, architecture documents, incident post-mortems, and code comments that explain WHY a suboptimal path was chosen.

The critical distinction: a tradeoff is a deliberate prioritization between two valid concerns. A hack that sacrifices one concern accidentally is debt, not a tradeoff.

## Where to look (priority order)

### 1. Architecture Decision Records

```text
Glob: **/adr/**/*.md
Glob: **/decisions/**/*.md
Glob: **/docs/architecture*.md
Glob: **/docs/design*.md
```

ADRs frequently document tradeoffs explicitly. Look for sections titled: "Trade-offs", "Consequences", "Alternatives considered", "What we gave up", "Downsides". The rejected alternatives reveal what was sacrificed.

Extract: what was chosen, what was sacrificed, and why the winner won.

### 2. Code comments explaining suboptimal choices

```text
Grep: "trade-?off"
Grep: "sacrifice"
Grep: "at the cost of"
Grep: "at the expense of"
Grep: "in exchange for"
Grep: "prioritize"
Grep: "prefer.*over"
Grep: "chose.*instead"
Grep: "accepted.*because"
Grep: "slower.*but"
Grep: "more complex.*but"
Grep: "less efficient.*but"
Grep: "not ideal.*but"
```

Comments that acknowledge a cost reveal an implicit tradeoff. "This is slower but ensures consistency" means consistency was prioritized over performance.

### 3. Configuration choices with rationale

```text
Grep: "consistency"
Grep: "availability"
Grep: "partition"
Grep: "CAP"
Grep: "eventual"
Grep: "strong.?consistency"
Grep: "read.?after.?write"
```

Database and infrastructure choices often encode fundamental tradeoffs (CAP theorem, latency vs consistency, cost vs redundancy).

### 4. Incident post-mortems and retrospectives

```text
Glob: **/postmortem*
Glob: **/post-mortem*
Glob: **/retrospective*
Glob: **/incident*
Grep: "root cause"
Grep: "we learned"
Grep: "going forward"
Grep: "action item"
```

Post-incident decisions often create tradeoffs: "We chose to add latency to the payment path in exchange for guaranteed idempotency after the double-charge incident."

### 5. Performance vs correctness patterns

```text
Grep: "cache"
Grep: "eventual"
Grep: "stale"
Grep: "TTL"
Grep: "invalidat"
Grep: "sync"
Grep: "async"
Grep: "queue"
Grep: "batch"
```

Caching, async processing, and batching are almost always tradeoffs: speed vs freshness, throughput vs latency, efficiency vs simplicity.

### 6. External sources

**Issue tracker**: Search for tickets about architectural decisions, performance investigations, and incident responses. Tickets often document WHY one approach was chosen over another.

**Pull requests**: Large PRs and RFCs often have discussion threads where tradeoffs are debated:

```bash
gh pr list --search "tradeoff OR trade-off OR architecture OR design" --state merged --limit 20
```

Read the discussion, not just the description.

### 7. Product and business tradeoffs

```text
Grep: "MVP"
Grep: "scope"
Grep: "defer"
Grep: "phase 2"
Grep: "v2"
Grep: "good enough"
Grep: "ship"
Grep: "iterate"
```

Product decisions encode tradeoffs between completeness and speed-to-market, between feature depth and breadth.

## Debt signals for tradeoffs

Classify as **debt**, not truth, when:

- The tradeoff references a constraint that no longer applies ("We accepted stale data because the database was slow" but the database was since upgraded)
- The losing side has grown more important than when the tradeoff was made but the code has not been updated
- The tradeoff was explicitly temporary ("For now we prioritize X over Y until we can do both")

Classify as **ambiguous** when:

- Code implies a priority (e.g., caching a value that could be computed fresh) but no documentation explains the choice
- Two patterns in the codebase imply opposite priorities in different areas

## Writing tradeoff candidates

State both sides, the winner, and the reasoning:

```yaml
# From ADR: "We chose eventual consistency for the product catalog
# because strong consistency added 200ms to every page load. Product
# data changes infrequently and staleness up to 30s is acceptable."

# Candidate:
name: "Catalog: latency over freshness"
content: "Product catalog data prioritizes read latency over
freshness. Stale data up to 30 seconds is acceptable because product
information changes infrequently. Page load speed was measured as
the primary driver of conversion, outweighing the cost of
occasionally showing outdated stock or pricing for a brief window."
context: "product catalog, caching, eventual consistency, read
performance, page load speed, conversion optimization -- chosen
after A/B test showed 200ms latency increase reduced conversion 3%"
classification: truth
confidence: high
reasoning: "Explicit ADR with measured data supporting the priority.
Caching configuration in production matches the stated 30s TTL."
```
