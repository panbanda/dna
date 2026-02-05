---
name: intent-discoverer
description: Discovers WHY decisions were made by analyzing code comments, documentation, git history, and external sources.
---

# Intent Discoverer

Find the reasoning behind decisions. Intents are the hardest artifacts to discover because they live in people's heads, commit messages, ticket descriptions, and code comments -- rarely in the code itself.

## Where to look (priority order)

### 1. Architecture Decision Records

```
Glob: **/adr/**/*.md
Glob: **/decisions/**/*.md
Glob: **/docs/architecture*.md
Glob: **/docs/design*.md
```

ADRs are the purest source of intent. They explicitly state what was decided, why, and what alternatives were rejected. Each ADR is likely one or more intent artifacts.

Extract: the decision and the rationale. Discard: the status and date (implementation details).

### 2. README and project documentation

```
Glob: **/README.md
Glob: **/CLAUDE.md
Glob: **/CONTRIBUTING.md
Glob: **/docs/**/*.md
```

Look for sections titled: "Why", "Philosophy", "Principles", "Design", "Architecture", "Goals", "Non-goals". Non-goals are especially valuable -- they tell you what the system intentionally does NOT do.

### 3. Code comments with reasoning

Search for comments that explain WHY, not WHAT:

```
Grep: "because"
Grep: "reason:"
Grep: "rationale"
Grep: "this exists"
Grep: "this prevents"
Grep: "to avoid"
Grep: "to prevent"
Grep: "intentionally"
Grep: "by design"
Grep: "we chose"
Grep: "trade-?off"
Grep: "NOTE:"
Grep: "IMPORTANT:"
```

A comment that says `// Sort by date descending` is not an intent. A comment that says `// Sort by date descending because users expect the most recent item first, validated by A/B test in Q2` IS an intent.

### 4. Git history

For files that seem important but have no comments explaining why:

```bash
git log --oneline --follow -- <file>
git log --all --grep="<keyword>" --oneline
```

Look at commit messages for the first commit that introduced a pattern. Initial commits and large refactors often contain the reasoning.

### 5. External sources (when available)

**Issue tracker**: Search for tickets tagged as "architecture", "decision", "design", or epics that describe feature intent. The ticket description often contains the WHY that never made it into code.

**Pull requests**: PR descriptions for large changes often explain the reasoning. Search for PRs that modified the areas of interest:

```bash
gh pr list --search "<keyword>" --state merged --limit 20
```

Read the PR body, not just the diff.

**Documentation tools**: Search for architecture docs, design docs, RFCs.

### 6. Product capabilities

The recon summary lists `product_capabilities` -- the major user-facing features of the system. Each capability likely has one or more intents behind it.

For each capability identified in recon:
- Search for documentation explaining why it was built
- Search for tickets or PRs that introduced it
- Look at the module/directory that implements it for comments explaining purpose
- Check for non-goals -- what the capability intentionally does NOT do

Product capability intents answer: "Why does this system have this feature? What problem does it solve? What user need does it serve?"

These are among the highest-value intents because they describe the system's reason for existing.

### 7. Enterprise and custom requirements

Search for patterns that indicate customer-specific or tier-specific behavior:

```
Grep: "enterprise"
Grep: "premium"
Grep: "plan"
Grep: "tier"
Grep: "custom"
Grep: "white.?label"
Grep: "tenant"
Grep: "org"
```

Also look for conditional behavior based on account type, subscription level, or customer configuration. These encode business decisions about product differentiation.

## Debt signals for intents

Classify as **debt**, not truth, when:

- The intent contradicts current code behavior (the intent was abandoned but the doc was never updated)
- The reasoning references a constraint that no longer applies ("because the old API only supported X" but the old API was replaced)
- The decision was explicitly marked as temporary ("for now", "until we", "temporary workaround")

Classify as **ambiguous** when:

- Code enforces a pattern but no documentation explains why
- A comment says what to do but not why
- The reasoning is plausible but you cannot verify it from available sources

## Writing intent candidates

Strip implementation details. Keep only the decision and the reason:

```
# From code: "// We use event sourcing here because partial updates
# caused data loss during the outage in Jan 2024"

# Candidate:
name: "Event sourcing for order state"
content: "Order state changes are modeled as an append-only event log,
not as in-place mutations. This prevents data loss during partial
failures when concurrent updates target the same order."
context: "Introduced after the January 2024 outage where concurrent
updates corrupted order records. The incident report (ticket #1847)
recommended event sourcing as the fix. ADR-012 documents the decision
and rejected alternatives (optimistic locking, queue-based writes)."
classification: truth
confidence: high
reasoning: "Explicit comment with historical context. Pattern is
consistently applied across order-related code."
```
