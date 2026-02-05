---
name: validating-approach
description: Validates a proposed implementation plan against DNA truth artifacts to catch conflicts before any code is written. Use after forming an approach but before beginning implementation. Checks contracts, constraints, and evaluations against each planned step.
---

# Validating an Approach

You have a plan. Before executing it, verify each step against stored truth.

## For each step in the plan

### Check contracts

```bash
dna search "<component or interface this step touches>" --kind contract
```

If a contract governs the component, verify:

- Does the step respect what the contract promises?
- Does the step depend on something the contract does not guarantee?
- Does the step change an interface that other consumers depend on?

### Check constraints

```bash
dna search "<the technique or pattern this step uses>" --kind constraint
```

If a constraint applies, verify:

- Does the step operate within the stated limits?
- Does the step introduce something the constraint prohibits?

### Check evaluations

```bash
dna search "<what this step is building>" --kind evaluation
```

If evaluations exist, they define what "correct" means. Your implementation must satisfy them.

### Check algorithms

```bash
dna search "<any computation this step involves>" --kind algorithm
```

If an algorithm artifact exists, the implementation must match its logic. Do not invent alternative formulas.

## Conflict resolution

If a step conflicts with stored truth:

1. **Stop.** Do not proceed with that step.
2. State the specific conflict: which artifact, which step, what the contradiction is.
3. Propose alternatives:
   - Modify the step to fit the truth
   - Propose updating the truth (with justification)
   - Split the step to isolate the conflicting part

## Validation summary

Before proceeding to implementation, state:

- How many truth artifacts were consulted
- Which artifacts constrain which steps
- Whether any conflicts were found (and how they were resolved)
- What new truth should be captured after execution
