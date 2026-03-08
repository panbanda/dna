---
name: writing-escalations
description: Creates escalation artifacts that define when an agent should stop autonomous work and involve a human. Use when encoding decision boundaries for agent autonomy -- the guard rails on what agents can decide alone.
---

# Writing Escalations

An escalation defines when an agent should stop autonomous work and involve a human. Distinct from constraints (hard limits) and pace (change velocity). Escalations are guard rails on agent autonomy itself.

## What an escalation IS

A decision boundary that tells an agent: "You may proceed autonomously up to this point. Beyond it, get a human." It answers: "When must a human be in the loop?"

## What an escalation is NOT

- Not a constraint ("Max upload size: 100MB") -- constraints limit the system, escalations limit the agent
- Not a pace artifact ("Auth model: slow") -- pace governs change velocity, escalations govern decision authority
- Not an intent ("Orders must not ship without payment") -- intents state rules, escalations state who decides

## Structure

An escalation must include:

1. **The trigger** -- what condition activates the escalation
2. **The threshold** -- specific and measurable where possible
3. **Who to escalate to** -- role or team, not a specific person
4. **What information to include** -- what the human needs to make the decision
5. **The reasoning** -- why autonomous action is inappropriate here

```text
Escalate to <who> when <trigger exceeds threshold>.
Include: <what context>. <Why autonomous action is inappropriate>.
```

## Examples

**Good:**

```bash
dna add escalation \
  "Escalate to engineering lead when estimated cost impact of a change exceeds \$1000. Include: the change description, affected systems, cost breakdown, and reversibility assessment. Autonomous cost decisions at scale compound errors -- a human must approve significant financial exposure." \
  --name "Cost impact threshold" \
  --label severity=high \
  --label domain=operations
```

Clear trigger, measurable threshold, specific audience, defined context.

```bash
dna add escalation \
  "Escalate to security team when a proposed change modifies authentication, authorization, or encryption logic. Include: the diff, affected endpoints, and which security properties change. Security model changes have blast radius beyond what an agent can assess from code alone." \
  --name "Security model changes" \
  --label severity=critical \
  --label domain=security
```

```bash
dna add escalation \
  "Escalate to product owner when a task requires choosing between two valid interpretations of a requirement and no tradeoff artifact resolves the ambiguity. Include: both interpretations, their implications, and which artifacts were consulted. Ambiguous requirements are a product decision, not a technical one." \
  --name "Ambiguous requirements" \
  --label severity=medium \
  --label domain=product
```

```bash
dna add escalation \
  "Escalate to data team when a schema migration would affect more than 5 tables or modify a column used by 3 or more services. Include: migration SQL, affected tables, dependent services, and estimated migration duration. Cross-service schema changes risk cascading failures." \
  --name "Large schema migrations" \
  --label severity=high \
  --label domain=data
```

**Bad:**

```text
Ask a human when unsure.
```

Too vague. Every agent is "unsure" about something. An escalation must define a specific trigger.

```text
Never make changes without approval.
```

This eliminates autonomy entirely. Escalations define the BOUNDARY of autonomy, not the absence of it.

## Severity levels

Use the `severity` label to indicate the impact level if the escalation is missed:

- **critical**: Autonomous action here could cause irreversible damage (data loss, security breach, legal liability)
- **high**: Significant financial, operational, or reputational impact
- **medium**: Incorrect decision is costly to reverse but not catastrophic
- **low**: Convenience escalation -- human input improves quality but autonomous action would not cause harm

## During planning

When forming a plan with autonomous execution steps:

1. Search for escalations: `dna search "<what the agent will do>" --kind escalation`
2. If an escalation applies, the plan must include a human checkpoint at that step
3. If no escalation exists for a high-risk step, consider whether one should be created

## Relationship to other kinds

- Escalations guard **intents**: when an intent is critical enough that violating it requires human judgment
- Escalations complement **tradeoffs**: when a tradeoff cannot resolve a conflict, the escalation says "ask a human"
- Escalations may sit **before** high-risk actions that approach important limits, but they do not replace **constraints**. If an action would violate a constraint, the agent must not proceed.
- Escalations inform **pace**: slow-pace areas often need escalations for any autonomous changes
