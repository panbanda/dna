---
name: writing-evaluations
description: Creates evaluation artifacts that define what success means with testable criteria. Use when establishing acceptance criteria, invariants, or verification benchmarks that determine whether an implementation is correct.
---

# Writing Evaluations

An evaluation defines what "correct" means. Without evaluations, an agent has no way to verify its own work against the system's actual requirements.

## What an evaluation IS

An executable test criterion: one invariant, scenario, or regression condition. It answers: "How do I know this is working correctly?"

## What an evaluation is NOT

- Not a vague quality goal ("should be reliable")
- Not a test command ("run pytest tests/")
- Not a feature description (that is an intent)

## Structure

Evaluations use the `--label type=<type>` convention:

- **invariant**: Something that must always be true
- **scenario**: A specific situation with expected behavior
- **regression**: A past failure that must not recur

## Examples

**Good -- invariant:**

```bash
dna add evaluation \
  "Account balance must never go negative. All withdrawal operations must check available balance before executing and reject with insufficient-funds if the withdrawal would result in a negative balance. This must hold under concurrent requests." \
  --name "Non-negative account balance" \
  --label type=invariant \
  --label area=billing
```

**Good -- scenario:**

```bash
dna add evaluation \
  "Authentication is correct when:
1. Valid credentials produce a session token within 500ms
2. Invalid credentials return a generic error that does not reveal whether the username or password was wrong
3. Sessions expire after 24h of inactivity
4. Expired sessions redirect to login without losing unsaved work
5. Concurrent sessions are permitted, up to 5 per user
6. Session 6 invalidates the oldest session" \
  --name "Authentication correctness" \
  --label type=scenario \
  --label area=auth
```

Each numbered criterion is independently verifiable. An agent writing tests knows exactly what to test.

**Good -- regression:**

```bash
dna add evaluation \
  "Timezone-dependent reports must produce identical output regardless of the server's local timezone. Verified by: generating the same report with TZ=UTC and TZ=US/Pacific and diffing the output. Regression from March 2024 where monthly reports showed different totals depending on which server generated them." \
  --name "Timezone-independent reports" \
  --label type=regression \
  --label area=reporting
```

The regression description includes the verification method and the original failure.

**Bad:**

```
The auth system should work correctly and be tested.
```

Not an evaluation. "Correctly" is undefined. "Tested" is a process, not a criterion.

```
Response time should be good.
```

Not measurable. An evaluation would be: "All customer-facing endpoints must respond within 200ms at p95 under a load of 100 concurrent users."

## Writing verifiable criteria

Each criterion should be something you could check without knowing the implementation:

- **Observable**: You can verify it from outside the system
- **Specific**: Only one interpretation of pass/fail
- **Independent**: Each criterion can be checked separately

```
# Bad: unverifiable
The system handles errors gracefully.

# Good: verifiable
When the payment provider returns a timeout:
1. The system retries once after 2 seconds
2. If the retry fails, the order is placed in pending-payment state
3. The user sees "Payment is being processed" (not an error)
4. A background job retries every 5 minutes for up to 1 hour
5. If all retries fail, the user is notified via email
```

## Failure modes

Include failure modes that must be handled. These are often where the most important evaluations live:

```bash
dna add evaluation \
  "When the identity provider is unavailable:
1. Existing sessions continue to work (cached validation)
2. New login attempts queue for up to 30 seconds
3. After 30 seconds, show a maintenance message with estimated recovery
4. No data is lost -- queued login attempts are retried when the provider returns" \
  --name "Auth provider outage handling" \
  --label type=scenario \
  --label area=auth
```

## During planning

Search for evaluations early -- they define your acceptance criteria:

```bash
dna search "<what you are building>" --kind evaluation
```

If evaluations exist, your implementation must pass them. If none exist, write evaluations before writing code.
