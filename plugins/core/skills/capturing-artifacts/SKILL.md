---
name: capturing-artifacts
description: Stores truth artifacts in DNA with correct kind, format, metadata, and semantic context. Use when codifying decisions, contracts, intents, or constraints that should persist across code changes, team transitions, and framework evolutions.
---

# Capturing Artifacts

Truth artifacts preserve system identity. Write them to survive a rewrite.

## Before you store

Verify the artifact passes these checks:

1. **Rewrite test**: Would this still be true if the system were rewritten in a different language? If no, you are capturing implementation, not identity.
2. **Two-agent test**: Could two different agents read this and make the same decisions? If no, it is too vague.
3. **Negative test**: Does it state what is NOT acceptable? Negative constraints are often more actionable than positive ones.
4. **Measurable test**: Where applicable, does it include measurable criteria? "Fast" is meaningless. "Under 50ms at p99" is actionable.
5. **Why test**: Does it explain WHY, not just WHAT? The reasoning prevents future agents from "fixing" intentional decisions.

## Choosing the right kind

| Kind | Use when | NOT for |
|------|----------|---------|
| intent | Capturing WHY something exists or works this way | Implementation details |
| contract | Defining BOUNDARIES between components | Internal implementation |
| constraint | Setting LIMITS on what is acceptable | Preferences or suggestions |
| algorithm | Describing HOW a computation works (the logic, not the code) | Code snippets |
| evaluation | Defining what SUCCESS means (testable criteria) | Vague quality goals |
| monitor | Specifying WHAT to observe and alert on | Tool-specific dashboards |
| glossary | Defining TERMS precisely to prevent ambiguity | Obvious vocabulary |
| integration | Describing HOW external systems connect | Internal service details |
| pace | Declaring CHANGE VELOCITY for an area (fast/medium/slow) | Sprint plans |
| compliance | Capturing REGULATORY requirements with source | General best practices |
| reporting | Defining WHAT must be queryable for business needs | Report layouts |

If unsure between two kinds, choose the one that answers the more fundamental question. "Why" (intent) is more fundamental than "how" (algorithm). "What are the limits" (constraint) is more fundamental than "what to observe" (monitor).

## Writing the content

### Structure

One artifact = one truth. Do not combine multiple decisions into a single artifact. "The billing system uses immutable invoices AND charges tax after discounts" is two artifacts.

### Specificity

State observable behavior, not implementation:

```text
# Bad: implementation-specific
Use bcrypt with cost factor 12.

# Good: observable behavior
Passwords must be hashed with a memory-hard algorithm.
Cost parameters must make brute-force infeasible on current hardware
(target: >100ms per hash). Raw passwords must never be logged,
stored in plaintext, or transmitted after initial receipt.
```

### Edge cases

Document edge cases with reasoning:

```text
Shipping is calculated on the pre-discount subtotal.
Reason: prevents gaming via high-discount items where the
discount exceeds the shipping savings.
```

The "reason" prevents a future agent from "fixing" this to use post-discount subtotal.

## Storing the artifact

```bash
dna add <kind> "<content>" \
  --name "<short descriptive name>" \
  --label area=<domain-area> \
  --label team=<owning-team> \
  --context "<additional semantic context for search>"
```

### Name

Short, descriptive. Used for human identification. "Immutable invoices after finalization" not "billing-rule-1".

### Labels

Use consistent label keys across the project:

- `area=<domain>`: billing, auth, onboarding, etc.
- `team=<owner>`: who is responsible for this truth
- `service=<service>`: which service this governs
- `regulation=<name>`: for compliance artifacts (gdpr, hipaa, pci-dss)

### Context

The `--context` flag adds semantic context for search without polluting the artifact content. Use it when the content alone might not surface for relevant queries:

```bash
dna add intent "Orders must not ship until payment is confirmed" \
  --name "Payment before shipment" \
  --label area=orders \
  --context "fulfillment, warehouse, payment verification, shipping hold"
```

### Format

Choose format based on content structure:

- `--format markdown`: Prose with structure (default, good for most artifacts)
- `--format yaml`: Structured data with clear fields
- `--format text`: Single atomic statement
- `--format openapi`: API contracts specifically
