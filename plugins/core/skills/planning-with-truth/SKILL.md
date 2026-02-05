---
name: planning-with-truth
description: Consults DNA truth artifacts during task planning to understand system identity before designing an implementation approach. Use at the start of any non-trivial task before choosing an architecture or decomposing work into steps. Searches for intents, contracts, and constraints that shape the solution space.
---

# Planning with Truth

Before choosing an approach, understand what this system believes about itself.

## Step 1: Broad search

Query DNA for truth related to the domain you are about to change.

```bash
dna search "<what you are building or changing>"
dna search "<the domain area>" --kind intent
dna search "<the domain area>" --kind contract
dna search "<the domain area>" --kind constraint
```

Also list recent artifacts in the area if labels are used:

```bash
dna list --kind intent --label area=<relevant-area>
```

You are looking for three things:

- **Intents**: WHY the system works this way. These constrain your goals.
- **Contracts**: BOUNDARIES between components. These constrain your interfaces.
- **Constraints**: LIMITS on what is acceptable. These constrain your approach.

## Step 2: Narrow the solution space

Your plan must fit within the identity you discovered. If a stored intent says "all auth flows go through the gateway," do not plan a solution that bypasses the gateway.

If multiple approaches are valid, prefer the one that aligns with the most stored truth without requiring exceptions.

## Step 3: Identify gaps

If there are no relevant artifacts, that is signal. It means this area has no codified truth yet. Your plan should include capturing the decisions you are about to make. See the [capturing-artifacts](../capturing-artifacts/SKILL.md) skill.

For each significant decision in your plan, ask: "Is this a new intent, contract, or constraint that should be stored?"

## Step 4: Surface conflicts early

If the task as requested conflicts with stored truth, state the conflict BEFORE proposing a plan. The options are:

1. Adjust the plan to fit the truth
2. Update the truth (requires explicit approval -- truth is not disposable)
3. Accept the conflict and document the exception with reasoning

Never silently override stored truth. If you proceed without surfacing a conflict, a future agent will find the inconsistency and not know which source to trust.

## What to include in your plan

After consulting DNA, your plan should reference:

- Which truth artifacts you found and how they shaped the approach
- Which artifacts constrain specific steps
- Where no truth exists and new artifacts should be captured
- Any conflicts between the requested task and stored truth
