---
name: writing-tradeoffs
description: Creates tradeoff artifacts that encode which concern wins when competing concerns clash. Use when two valid goals, constraints, or intents conflict and agents need to know which to prioritize.
---

# Writing Tradeoffs

A tradeoff encodes which concern wins when competing concerns clash. Without tradeoffs, agents that find conflicting artifacts have no way to know which to sacrifice.

## What a tradeoff IS

A priority declaration between two or more competing concerns, with conditions and reasoning. It answers: "When X and Y conflict, which wins?"

## What a tradeoff is NOT

- Not a preference ("We prefer fast deploys") -- preferences without stakes are too soft
- Not a constraint ("Latency < 200ms") -- constraints are hard limits, not priorities between limits
- Not an intent ("Users must verify email") -- intents are standalone decisions, not conflict resolutions

## Structure

A tradeoff must include:

1. **The competing concerns** -- name both sides explicitly
2. **The winner** -- which concern takes priority
3. **The conditions** -- under what circumstances this priority applies
4. **The reasoning** -- why this ordering exists (prevents future reversal without understanding the cost)

```text
<Higher concern> over <lower concern>: <what this means in practice>.
<Why this ordering>.
```

## Examples

**Good:**

```bash
dna add tradeoff \
  "Customer satisfaction over resolution speed: never auto-close a support ticket without confirming the customer's issue is resolved, even if it means longer average resolution times. Klarna's AI optimized for resolution time because it was measurable, but customers left because their problems were not actually solved." \
  --name "Satisfaction over speed" \
  --label domain=support
```

Both concerns are valid (fast resolution, happy customers). The tradeoff says which wins and why.

```bash
dna add tradeoff \
  "Billing correctness over latency: billing calculations must produce provably correct results even if the computation takes longer than the standard API latency target. An incorrect invoice destroys trust and creates legal liability. The p95 latency constraint does not apply to billing endpoints." \
  --name "Billing correctness over latency" \
  --label domain=billing
```

This resolves a real conflict: the system has a latency constraint, but billing accuracy is more important.

```bash
dna add tradeoff \
  "Data consistency over availability: when a network partition occurs, the system rejects writes rather than accepting writes that might conflict. Financial data that diverges across nodes is worse than temporary unavailability." \
  --name "Consistency over availability" \
  --label domain=infrastructure
```

**Bad:**

```text
Speed is important but so is quality.
```

This states the tension without resolving it. A tradeoff must pick a winner.

```text
We value security.
```

This is not a tradeoff. A tradeoff would be: "Security over developer velocity: all dependency updates require security review even though it slows releases by 2 days."

## The Klarna test

Before storing a tradeoff, apply the Klarna test: if an AI agent optimized for the LOSING side of this tradeoff, what would go wrong? If you cannot articulate the damage, the tradeoff may not be worth storing.

## During planning

When you encounter conflicting artifacts during planning:

1. Search for an existing tradeoff: `dna search "<the conflict>" --kind tradeoff`
2. If one exists, your plan must respect the stated priority
3. If none exists, surface the conflict to the user and propose a tradeoff to capture

## Relationship to other kinds

- Tradeoffs resolve conflicts between **intents**: when two intents pull in opposite directions
- Tradeoffs can override **constraints**: a constraint may be relaxed for a higher concern
- Tradeoffs inform **evaluations**: the winning concern should have evaluations; the losing concern may have softer targets
- Tradeoffs complement **escalations**: when neither side can clearly win, the tradeoff may say "escalate to human"
