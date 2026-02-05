---
name: writing-intents
description: Creates intent artifacts that capture WHY a system works the way it does. Use when codifying a decision about system behavior, a business rule, or a design principle that should survive code changes and framework evolutions.
---

# Writing Intents

An intent captures WHY something exists or works a particular way. It is the most fundamental kind of truth -- everything else (contracts, constraints, algorithms) flows from intents.

## What an intent IS

A declarative statement about a user-observable outcome or rule. It answers: "Why does the system behave this way?"

## What an intent is NOT

- Not an implementation detail ("We use Redis for caching")
- Not a task ("Add password hashing to the login endpoint")
- Not a wish ("The system should be fast")
- Not a constraint (constraints have measurable limits; intents have reasons)

## Structure

One intent = one decision. State the rule, then the reason.

```text
<What must be true>. <Why this matters or what goes wrong without it>.
```

## Examples

**Good:**

```bash
dna add intent \
  "Orders must not ship until payment is confirmed. Shipping unconfirmed orders caused $40K in losses in Q2 from chargebacks on fraudulent orders." \
  --name "Payment before shipment" \
  --label domain=orders
```

The "because" makes this intent defensible. A future agent will not try to add an "express ship without payment" feature.

```bash
dna add intent \
  "The billing system treats invoices as immutable after finalization. Corrections are modeled as new adjustment invoices that reference the original. Partial mutations caused reconciliation failures that took 2 weeks to untangle." \
  --name "Immutable invoices" \
  --label domain=billing
```

This explains not just the rule but the cost of violating it.

```bash
dna add intent \
  "Every user-facing error message must be actionable. It must tell the user what went wrong and what they can do about it. Generic error messages generate support tickets." \
  --name "Actionable error messages" \
  --label domain=ux
```

**Bad:**

```text
We use PostgreSQL.
```
This is a technology choice, not an intent. The intent would be: "The system requires ACID transactions with complex relational queries. Document stores were evaluated and rejected because of cross-entity consistency requirements."

```text
The API should be RESTful.
```
This is an architectural preference. The intent would be: "External consumers depend on stable, predictable URLs. Breaking URL changes have historically caused integration failures with 3+ partners."

## During planning

When forming a plan and you encounter a decision about WHY:

1. Search for an existing intent: `dna search "<the decision>" --kind intent`
2. If one exists, your plan must align with it
3. If none exists, include capturing a new intent as part of your plan

## Relationship to other kinds

- Intent drives **contracts**: "All auth goes through the gateway" (intent) leads to "The gateway exposes /auth/token" (contract)
- Intent drives **constraints**: "We must support offline usage" (intent) leads to "Client must function without network for up to 24 hours" (constraint)
- Intent drives **algorithms**: "Pricing must account for volume discounts" (intent) leads to the specific discount formula (algorithm)

If you find yourself writing implementation details in an intent, you probably need an algorithm or contract instead.
