---
name: writing-contracts
description: Creates contract artifacts that define boundaries between components -- what they promise, what they expect, and what they explicitly exclude. Use when establishing or documenting how services, modules, or systems interact.
---

# Writing Contracts

A contract defines a BOUNDARY between components. It captures what one side promises and what the other side can depend on.

## What a contract IS

An external promise: one endpoint, event, interface, or integration point. It answers: "What can consumers depend on?"

## What a contract is NOT

- Not internal implementation details
- Not the full API schema (that changes; the contract survives)
- Not a description of current behavior (it is a statement of guaranteed behavior)

## Structure

A contract must include:

1. **What it promises** (outputs, guarantees)
2. **What it requires** (inputs, preconditions)
3. **What it does NOT provide** (explicit exclusions)
4. **Failure modes** callers must handle
5. **Performance bounds** where applicable

## Examples

**Good:**

```bash
dna add contract \
  "The user service exposes identity lookup. Callers provide an opaque user identifier. The service returns display identity (name, avatar) and contact identity (email). The service NEVER exposes authentication credentials or internal database identifiers. Callers must handle: not-found, rate-limited, unavailable. Response latency must stay under 50ms at p99." \
  --name "User identity lookup" \
  --label area=users \
  --label service=user-service
```

This survives a rewrite from REST to GraphQL. It tells consumers what they can depend on and what they cannot.

```bash
dna add contract \
  "The event bus guarantees at-least-once delivery. Consumers must be idempotent. Message ordering is guaranteed within a partition key but not across partition keys. Maximum message size is 1MB. Messages not acknowledged within 30 seconds are redelivered." \
  --name "Event bus delivery guarantees" \
  --label area=infrastructure \
  --label service=event-bus
```

This defines the contract without naming Kafka, RabbitMQ, or any specific technology.

**Bad:**

```
GET /api/v2/users/:id returns { id: number, name: string, email: string }
```

This is a schema snapshot, not a contract. It will be stale when you add a field. The contract should describe what categories of data are returned, not exact field names.

```
The user service is built with Express.js and uses MongoDB.
```

This is implementation, not a contract. Consumers should not know or care.

## The "negative contract"

What a contract explicitly excludes is often more valuable than what it includes. Stating "NEVER exposes authentication credentials" prevents a future agent from adding a convenience endpoint that leaks tokens.

Always ask: "What must this component NOT do, even if it would be convenient?"

## Format choice

- **Markdown** (default): Good for prose contracts with nuance
- **OpenAPI**: Good for HTTP API contracts specifically -- use `--format openapi`
- **YAML**: Good for structured contracts with clear fields

For API contracts, consider storing both: an OpenAPI spec for the machine-readable shape, and a markdown contract for the guarantees that OpenAPI cannot express (latency bounds, semantic invariants, failure mode expectations).

## During planning

When your plan introduces or modifies an interface:

1. Search for existing contracts: `dna search "<the interface>" --kind contract`
2. If a contract exists, verify your plan respects it
3. If you are creating a new interface, draft the contract before implementing
