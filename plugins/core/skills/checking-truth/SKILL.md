---
name: checking-truth
description: Queries DNA truth artifacts before modifying a specific file or component. Use before any code change to verify it does not conflict with established intents, contracts, or constraints. Lighter-weight than full plan validation -- operates at the individual change level.
---

# Checking Truth

Before modifying code, check what truth governs this area.

## Quick check

```bash
dna search "<what you are about to change>" --limit 5
```

Review the results. If any artifact has a similarity score above 0.6, read it carefully. It likely governs what you are about to do.

## Targeted checks

If the change modifies an interface or API:

```bash
dna search "<the interface>" --kind contract
```

If the change modifies business logic or computations:

```bash
dna search "<the computation>" --kind algorithm
```

If the change touches a constrained area (security, performance, compliance):

```bash
dna search "<the area>" --kind constraint
dna search "<the area>" --kind compliance
```

## Interpreting results

| Score | Meaning | Action |
|-------|---------|--------|
| > 0.8 | Directly relevant | Read and comply |
| 0.6 - 0.8 | Likely relevant | Read and assess |
| 0.4 - 0.6 | Possibly related | Skim for applicability |
| < 0.4 | Probably noise | Ignore |

## When truth conflicts with your change

Do not silently proceed. State the conflict:

- What you intended to change
- What the artifact says
- Whether you recommend adjusting the change or updating the artifact

## When no truth exists

If you are making a significant decision and no truth covers it, note this. After the change, consider capturing the decision. See [capturing-artifacts](../capturing-artifacts/SKILL.md).
