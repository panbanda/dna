---
name: reviewing-compliance
description: Reviews current code or proposed changes against stored DNA truth artifacts to identify violations, drift, and uncovered areas. Use after implementation to verify alignment with intents, contracts, and constraints, or during audits to assess truth coverage.
---

# Reviewing Compliance

Check whether reality matches stored truth.

## After implementation

Once changes are made, verify they align with the artifacts that were consulted during planning.

### Step 1: Re-check consulted artifacts

For each artifact that was referenced during planning:

```bash
dna get <artifact-id>
```

Verify the implementation satisfies the artifact. Specifically:

- **Intents**: Does the implementation serve the stated purpose?
- **Contracts**: Does the implementation honor the promised interface?
- **Constraints**: Does the implementation stay within stated limits?
- **Algorithms**: Does the implementation match the stated logic?
- **Evaluations**: Does the implementation pass the stated criteria?

### Step 2: Check for unintended violations

Search for artifacts that were NOT consulted but might be affected:

```bash
dna search "<what was changed>" --limit 10
```

Review any results with score > 0.5. A change to the billing service might violate a compliance artifact that was not part of the original plan.

### Step 3: Identify truth drift

Truth drift happens when the system has changed but the artifacts have not been updated, or when artifacts exist that no longer match reality.

Signs of drift:

- An artifact describes behavior the system no longer exhibits
- An artifact references components that no longer exist
- Multiple artifacts contradict each other

When you find drift, do not silently ignore it. Either update the artifact or flag it for review.

## Coverage audit

To assess how well truth covers the system:

```bash
dna list --kind intent
dna list --kind contract
dna list --kind constraint
```

For each major system component, check whether:

- At least one intent explains why it exists
- Contracts define its boundaries with other components
- Constraints set its operational limits

Areas with no truth coverage are areas where agents will make unconstrained decisions. This is not always wrong (not everything needs to be codified), but high-risk areas (auth, billing, compliance, data handling) should have coverage.

## Capturing new truth

During review, you may discover decisions that were made but never stored:

- An implicit contract between services that was never documented
- A constraint that everyone "just knows" but is not recorded
- An algorithm that exists only in code with no specification

Capture these. See [capturing-artifacts](../../core/skills/capturing-artifacts/SKILL.md).

## Updating stale truth

When truth is stale:

```bash
dna update <artifact-id> --content "<updated content>"
```

Include a reason for the update in the content itself, or add a label:

```bash
dna update <artifact-id> \
  --content "<updated content>" \
  --label updated-reason=<brief-reason>
```

DNA preserves version history, so the original truth is not lost.
