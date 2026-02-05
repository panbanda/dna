# Extract Truth

Extract truth artifacts from arbitrary input -- documents, prompts, conversations, specs, or any non-code source.

## When to use

Use `extract-truth` when the source is NOT a codebase. For codebase analysis, use `discover-truth`.

Examples:
- A design document or architecture spec
- Product requirements (PRD, RFC, user stories)
- Meeting notes where decisions were captured
- An incident report or post-mortem
- A conversation transcript or interview notes
- An existing truth directory from another project
- Inline text from the user describing their system

## Prerequisites

1. DNA must be initialized: `dna init --template intent`
2. The user must provide the input source.

## Workflow

### Step 1: Identify the input

Ask the user what to extract from. Accept any of:

- **Inline text**: The user types or pastes content directly
- **File path**: A local file (markdown, YAML, JSON, plain text, PDF)
- **Multiple files**: A glob pattern or directory (`docs/*.md`, `truth/`)
- **URL**: A web page to fetch (use WebFetch)
- **MCP tool output**: Query a tool (e.g., search a wiki, fetch a ticket) and extract from the result

If the user provides a directory or glob, read all matching files and treat them as a single combined input.

### Step 2: Read and understand the input

Read the full input. Before extracting anything, understand:

- What is this document about? What system or domain does it describe?
- What is the document's purpose? (decision record, requirements spec, incident report, design doc, meeting notes)
- What level of authority does it carry? (approved spec vs. brainstorm notes vs. post-mortem findings)

The document type affects confidence:

| Document type | Default confidence | Rationale |
|---------------|-------------------|-----------|
| ADR, approved spec, signed-off PRD | high | Explicit decisions with authority |
| Design doc, RFC (accepted) | high | Reviewed and accepted |
| Design doc, RFC (draft/proposed) | medium | Not yet approved |
| Incident report, post-mortem | high for findings, medium for action items | Findings are observed truth; action items are proposed changes |
| Meeting notes | medium | Decisions may be informal or misrecorded |
| Conversation transcript | low | High noise, context often missing |
| User-provided description | medium | Depends on the user's authority and specificity |

### Step 3: Extract candidates

Read through the input and identify truth artifacts. For each candidate, determine:

1. **Kind**: Which artifact kind best fits? Use the same kinds as `discover-truth`:

   | Kind | What to look for in the input |
   |------|-------------------------------|
   | intent | Decisions and their reasoning. "We chose X because Y." Non-goals. Purpose statements. |
   | contract | Promises to users or other systems. API guarantees. SLAs. "The system will always..." |
   | constraint | Hard limits. "Must not exceed." "Never allow." Security requirements. |
   | algorithm | Computation rules. Formulas. Scoring logic. Business rules with specific numbers. |
   | evaluation | Success criteria. Acceptance criteria. "The feature is correct when..." |
   | integration | External dependencies. "We use X for Y." Vendor decisions. |
   | glossary | Domain terms defined or clarified. "By 'member' we mean..." |
   | monitor | Observability requirements. SLOs. Alert thresholds. "Page when..." |
   | compliance | Regulatory obligations. "GDPR requires..." "SOC2 mandates..." |
   | pace | Change velocity decisions. "Auth changes require security review." |
   | reporting | Business queries. KPIs. "Monthly report must show..." |

2. **Content**: Write the artifact content. Same rules as `discover-truth`:
   - Language-agnostic (no code, no framework names)
   - Survives a rewrite
   - Describes WHAT and WHY, not HOW

3. **Context**: Domain concepts, related areas, brief provenance. Creates graph-like connections in DNA's vector index.

4. **Classification**: For non-code sources, classification works differently:

   - **truth**: The document explicitly states this as a decision, requirement, or finding with authority
   - **debt**: The document identifies this as a known problem, a gap, or something that needs to change
   - **ambiguous**: The document mentions this but it is unclear whether it is a current requirement or aspirational

5. **Source**: Reference back to the input:
   - For files: `filename:section` or `filename:line`
   - For URLs: the URL
   - For inline text: `user-provided`
   - For MCP tool output: the tool and query used

### Step 4: Present candidates for review

Before storing, present all candidates to the user grouped by classification:

```
EXTRACTED TRUTH CANDIDATES:

  [intent] "Event sourcing for order state"
  [intent] "Multi-region deployment for latency"
  [contract] "API backwards compatibility guarantee"
  [constraint] "No PII in log output"
  ...

POTENTIAL DEBT IDENTIFIED:
  [constraint] "MD5 still used for legacy checksums"
  ...

AMBIGUOUS -- NEEDS YOUR INPUT:
  [evaluation] "99.9% uptime target" -- Is this an active SLA or aspirational?
  ...

Store N truth artifacts? [y/n]
```

Let the user review, remove candidates, reclassify, or edit before storing.

### Step 5: Store

For each approved candidate:

```bash
dna add <kind> "<content>" \
  --name "<name>" \
  --label source=<document-type> \
  --label extracted=true \
  --label confidence=<confidence> \
  --context "<context>"
```

Use `extracted=true` instead of `discovered=true` to distinguish artifacts from manual extraction vs. codebase discovery.

### Step 6: Summary

```
TRUTH ARTIFACTS STORED: N
  (list each with kind and name)

Source: <input description>

DEBT FINDINGS: N
  (list each -- consider creating tickets)

SKIPPED: N
  (items the user chose not to store)
```

## Handling different input types

### Design documents and specs

These are the richest source. A single design doc may contain intents (decisions), contracts (promises), constraints (limits), algorithms (formulas), and evaluations (acceptance criteria). Extract all kinds.

### Incident reports and post-mortems

Focus on:
- **Constraints** discovered during the incident ("the system cannot handle more than X")
- **Intents** from action items that become permanent decisions
- **Evaluations** that would have caught the issue ("we now verify X")
- **Monitors** added as a result

Be careful: action items that say "we should" are not yet truth. They are proposed changes. Classify as ambiguous unless the user confirms they were implemented.

### Product requirements (PRD, user stories)

Focus on:
- **Intents** for each feature and its purpose
- **Contracts** for user-facing guarantees
- **Evaluations** from acceptance criteria
- **Constraints** from non-functional requirements

### Meeting notes and conversations

High noise. Focus on:
- Statements that start with "we decided," "the decision is," "going forward we will"
- Explicitly stated non-goals
- Numbers and thresholds that were agreed on

Classify most findings as medium or low confidence unless the meeting notes explicitly mark decisions as final.

### Existing truth directories

If the input is a structured truth directory (like `truth/intent/*.md`, `truth/contracts/openapi.yaml`), the structure itself tells you the kind. Read each file and convert to DNA artifacts, preserving the original structure as context.
