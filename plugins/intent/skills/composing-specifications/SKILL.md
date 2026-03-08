---
name: composing-specifications
description: Assembles DNA truth artifacts into a self-contained specification that an autonomous agent can execute against without further human guidance. Use when handing off a well-defined deliverable to an agent for extended autonomous execution.
---

# Composing Specifications

A specification is a self-contained document that gives an autonomous agent everything it needs to execute a deliverable without further human guidance. It is the bridge between stored truth and autonomous action.

Composing specifications complements planning-with-truth:
- **planning-with-truth** = interactive planning (understand the solution space WITH a human)
- **composing-specifications** = autonomous handoff (assemble everything an agent needs to work ALONE)

## When to compose a specification

- The deliverable is well-defined and scoped to a single outcome
- Sufficient truth artifacts exist to bound the agent's decisions
- The work will run for an extended period without human interaction
- You want to maximize the quality of autonomous output by front-loading context

Do NOT compose a specification when:
- The problem space is still being explored (use planning-with-truth instead)
- Critical tradeoffs or escalations are missing for the domain
- The deliverable requires ongoing human judgment at multiple points

## Step 1: Define scope

One specification = one deliverable. If the work produces multiple independent outcomes, compose multiple specifications.

State the deliverable in one sentence:

```text
Build/Change/Fix <what> so that <observable outcome>.
```

## Step 2: Gather artifacts by role

Search DNA for artifacts that serve each role in the specification. Not every role will have artifacts -- gaps are identified in Step 3.

### WHY -- Intents

```bash
dna search "<the deliverable>" --kind intent
dna search "<the domain area>" --kind intent
```

Intents tell the agent why this work matters and what decisions have already been made. They prevent the agent from undermining existing system identity.

### BOUNDARIES -- Contracts and constraints

```bash
dna search "<the deliverable>" --kind contract
dna search "<the deliverable>" --kind constraint
```

Contracts define what the agent must not break. Constraints define the limits the agent must work within.

### PRIORITIES -- Tradeoffs

```bash
dna search "<the deliverable>" --kind tradeoff
dna search "<the domain area>" --kind tradeoff
```

Tradeoffs tell the agent what to sacrifice when competing concerns clash. Without tradeoffs, the agent will optimize for whatever is easiest to measure.

### DECISION BOUNDARIES -- Escalations

```bash
dna search "<the deliverable>" --kind escalation
dna search "<the domain area>" --kind escalation
```

Escalations tell the agent when to stop and involve a human. These are non-negotiable checkpoints in autonomous execution.

### ACCEPTANCE CRITERIA -- Evaluations

```bash
dna search "<the deliverable>" --kind evaluation
dna search "<the domain area>" --kind evaluation
```

Evaluations define what "done" means. Without evaluations, the agent has no way to know if its output is correct.

### DOMAIN CONTEXT -- Glossary, algorithms, compliance

```bash
dna search "<the deliverable>" --kind glossary
dna search "<the deliverable>" --kind algorithm
dna search "<the deliverable>" --kind compliance
```

Domain context prevents misunderstanding. Glossary terms ensure the agent uses words correctly. Algorithms ensure it computes correctly. Compliance ensures it does not violate regulations.

## Step 3: Identify gaps

Review what you found in Step 2 and flag gaps:

| Gap | Risk |
|-----|------|
| No evaluations for the deliverable | Agent has no acceptance criteria -- output quality is unverifiable |
| No escalations in a high-risk domain | Agent may make irreversible decisions autonomously |
| No tradeoffs where intents conflict | Agent will guess which intent to sacrifice |
| No constraints for the domain | Agent has no guardrails on approach |
| No glossary for domain-specific terms | Agent may misinterpret requirements |

**Critical gaps** (missing evaluations or missing escalations in high-risk domains) should be resolved before composing the specification. Either create the missing artifacts or accept the risk explicitly.

**Non-critical gaps** should be noted in the specification's Gaps section so the agent knows where its knowledge is incomplete.

## Step 4: Compose the specification

Assemble the gathered artifacts into a structured document with 8 sections.

### Section 1: Problem Statement

One paragraph. What is being built/changed/fixed and the observable outcome. Derived from the scope definition in Step 1.

### Section 2: System Beliefs

List the intents that govern this domain. These are facts the agent must not contradict. Present each as a declarative statement.

### Section 3: Boundaries

List the contracts and constraints that bound the work. Contracts define interfaces that must not break. Constraints define limits that must not be exceeded.

### Section 4: Optimization Targets

List the tradeoffs that apply. For each, state which concern wins and under what conditions. If no tradeoffs exist, state: "No tradeoffs are codified for this domain. When competing concerns arise, escalate."

### Section 5: Decision Boundaries

List the escalations that apply. These are non-negotiable: the agent MUST stop and involve a human when any of these triggers fire. If no escalations exist in a high-risk domain, this must be flagged as a critical gap.

### Section 6: Acceptance Criteria

List the evaluations that define success. These are the tests the agent must satisfy. Separate into invariants (always true), scenarios (specific cases), and regressions (must not recur).

### Section 7: Domain Context

Include relevant glossary terms, algorithms, and compliance requirements. Only include what is directly relevant to the deliverable.

### Section 8: Known Gaps

List what is NOT covered by stored truth. For each gap, state the risk and the recommended mitigation (usually: "use conservative defaults" or "escalate if uncertain").

## Step 5: Validate completeness

Before handing the specification to an agent, verify:

- [ ] Every evaluation has a supporting intent (you know WHY the acceptance criteria exists)
- [ ] No unresolved conflicts exist between intents without a tradeoff to resolve them
- [ ] Critical escalations are included for high-risk operations
- [ ] All domain-specific terms used in the spec are defined in the glossary section
- [ ] The specification can be understood without reading any code

## Output format

The composed specification should be stored or provided as a single document. It is NOT a DNA artifact -- it is a working document derived from artifacts.

```bash
# Optionally store the spec for reference
dna add intent \
  "Specification composed for <deliverable> on <date>. Covers: <list of sections>. Gaps: <list of gaps>." \
  --name "Spec: <deliverable>" \
  --label domain=<domain> \
  --label type=specification
```
