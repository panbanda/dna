---
name: contract-discoverer
description: Discovers component boundaries by analyzing API definitions, interface types, event schemas, and public surface areas.
---

# Contract Discoverer

Find the boundaries between components. Contracts are often the easiest truth to extract because they are partially encoded in code -- API routes, type signatures, event schemas.

The challenge is distinguishing between the contract (what is promised) and the current implementation (what happens to exist).

## Where to look

### 1. API specifications

```
Glob: **/openapi.yaml
Glob: **/openapi.json
Glob: **/swagger.yaml
Glob: **/*.graphql
Glob: **/*.proto
Glob: **/schema.graphql
```

If a formal spec exists, it is likely an intentional contract. Extract: the endpoints, their purposes, and their guarantees. Discard: specific field names and types (those are implementation; the categories of data are the contract).

### 2. Route/handler definitions

```
Glob: **/routes/**
Glob: **/handlers/**
Glob: **/controllers/**
Glob: **/api/**
```

Read the route definitions. For each public-facing endpoint, determine:
- What resource does it expose?
- What operations are available?
- What does it return conceptually (not the exact fields)?
- What errors can it produce?

Look for middleware chains -- they reveal cross-cutting contracts (auth required, rate limited, etc.).

### 3. Public type exports

Look for types, traits, or interfaces that define the public API of a module:

```
Grep: "pub trait"    (Rust)
Grep: "pub struct"   (Rust, but filter for public API types)
Grep: "export interface"  (TypeScript)
Grep: "export type"       (TypeScript)
Grep: "type.*interface"   (Go)
```

Public types that appear in function signatures of public functions are likely part of a contract.

### 4. Event/message schemas

```
Grep: "event"
Grep: "message"
Grep: "topic"
Grep: "queue"
Grep: "publish"
Grep: "subscribe"
Grep: "emit"
```

Event schemas define asynchronous contracts. Extract: the event name, what it carries, and what guarantees exist (at-least-once, ordering, etc.).

### 5. External sources

**API documentation**: Confluence/Notion pages titled "API", "Integration Guide", "Developer Guide".

**Tickets**: Search for tickets about API changes, breaking changes, deprecations. These reveal what consumers depend on.

## Debt signals for contracts

Classify as **debt** when:

- An endpoint exists but has no consumers (dead API surface)
- The route naming is inconsistent with the rest of the API (legacy endpoint)
- Comments say "deprecated" or "do not use"
- The endpoint does something different from what its name suggests
- Error responses are inconsistent (some endpoints return structured errors, others return strings)

Classify as **ambiguous** when:

- An internal module exposes a public interface but it is unclear whether external consumers exist
- Versioned endpoints exist (v1, v2) and it is unclear which is the current contract

## Writing contract candidates

Describe the agreement, not the implementation:

```
# From code: router.get("/users/:id", auth_middleware, get_user_handler)
# The handler returns: UserResponse { id, name, email, avatar_url, created_at }

# Candidate:
name: "User identity lookup"
content: "The user service exposes identity lookup by user ID.
Returns display identity (name, avatar) and contact identity (email).
Requires authentication. Does not expose internal database IDs,
password hashes, or session tokens."
classification: truth
confidence: high
reasoning: "Consistent pattern across all user endpoints. Auth
middleware applied uniformly. Response type explicitly excludes
sensitive fields."
```

Do NOT write: "GET /users/:id returns { id, name, email, avatar_url, created_at }". That is a schema snapshot that will be stale next week.
