---
name: writing-constraints
description: Creates constraint artifacts that set hard limits on what is acceptable -- performance bounds, security requirements, architectural boundaries. Use when establishing guardrails that must not be violated regardless of implementation approach.
---

# Writing Constraints

A constraint defines a LIMIT. It is the thing that turns "you could do anything" into "you must do it within these bounds."

## What a constraint IS

A technical, security, or operational limit that cannot be crossed. It answers: "What are the hard boundaries?"

## What a constraint is NOT

- Not a preference ("We prefer TypeScript" -- that is a decision/intent)
- Not a goal ("We want low latency" -- that is too vague to be a constraint)
- Not a current measurement ("Our p99 is 200ms" -- that is an observation, not a limit)

## Structure

A constraint must include:

1. **The limit** -- specific and measurable where possible
2. **The scope** -- what it applies to
3. **The reason** -- why this limit exists (prevents future relaxation without understanding the cost)

## Examples

**Good:**

```bash
dna add constraint \
  "User-provided content must never be interpreted as code, query syntax, or template expressions in any context. All rendering of user content must use contextual output encoding appropriate to the target format. This applies to every output channel including error messages, logs, and admin dashboards." \
  --name "No user content as code" \
  --label domain=security
```

An agent in any language reads this and knows: parameterize queries, encode output, no eval. It does not say "use DOMPurify" because the constraint survives a language change.

```bash
dna add constraint \
  "API response latency must stay under 200ms at p95 and under 500ms at p99 for all customer-facing endpoints. Background processing endpoints are excluded. This exists because the mobile app has a 2-second timeout and must complete at least 3 sequential API calls during checkout." \
  --name "API latency bounds" \
  --label domain=performance
```

The reason explains why these specific numbers matter, not just that they exist.

```bash
dna add constraint \
  "The system must run stateless for horizontal scaling. No request may depend on server-local state from a previous request. Session data must be stored in a shared backend, not in process memory. This is required because the auto-scaler can terminate any instance at any time." \
  --name "Stateless for horizontal scaling" \
  --label domain=architecture
```

**Bad:**

```text
Don't use eval().
```

Too implementation-specific. The constraint is "user input must never be interpreted as code." The eval() prohibition is one consequence of that constraint in one language.

```text
Be careful with security.
```

Not a constraint. This is an aspiration that constrains nothing.

```text
Max upload size: 100MB.
```

Close, but missing scope and reason. Better: "Max upload size is 100MB for all file upload endpoints. This exists because the Lambda runtime has a 6GB /tmp limit and we must support processing up to 50 concurrent uploads."

## Writing negative constraints

Negative constraints (what must NOT happen) are often more useful than positive ones:

- "Must never store raw passwords" -- unambiguous
- "Must never expose internal IDs in public APIs" -- clear boundary
- "Must never make synchronous calls to external services during request handling" -- architectural guardrail

These are easy to verify: either the violation exists or it does not.

## During planning

Constraints are the first thing to check when planning. They eliminate approaches before you waste time on them.

```bash
dna search "<your planned approach>" --kind constraint
```

If a constraint rules out your approach, you need a different approach -- not an exception to the constraint.
