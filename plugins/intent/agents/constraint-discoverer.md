---
name: constraint-discoverer
description: Discovers hard limits by analyzing validation logic, error handling, security middleware, configuration bounds, and infrastructure constraints.
---

# Constraint Discoverer

Find the hard boundaries. Constraints are encoded in validation functions, config limits, middleware, error handlers, and infrastructure.

The critical distinction: a constraint is a limit that SHOULD exist. A workaround or hack that imposes a limit is debt, not a constraint.

## Where to look

### 1. Input validation

```
Grep: "validate"
Grep: "max_length"
Grep: "min_length"
Grep: "max_size"
Grep: "MaxSize"
Grep: "limit"
Grep: "too_long"
Grep: "too_large"
Grep: "too_many"
Grep: "exceeds"
Grep: "overflow"
```

Every validation function reveals a constraint. The question is whether the limit is intentional or arbitrary. Look for:
- Limits with comments explaining why (intentional)
- Limits that match external requirements (intentional: "PCI requires", "GDPR mandates")
- Limits that are round numbers with no explanation (possibly arbitrary: `max: 100`)
- Limits that match infrastructure constraints (intentional: "Lambda has 6GB /tmp")

### 2. Security middleware and configuration

```
Grep: "auth"
Grep: "middleware"
Grep: "cors"
Grep: "csp"
Grep: "rate.limit"
Grep: "helmet"
Grep: "csrf"
Grep: "sanitize"
Grep: "escape"
Grep: "encrypt"
Grep: "hash"
```

Security patterns are almost always intentional constraints. Extract the policy, not the tool:
- "All endpoints require authentication" (not "uses passport.js")
- "CORS allows only the production domain" (not "cors({ origin: 'https://app.example.com' })")

### 3. Error handling patterns

```
Grep: "reject"
Grep: "deny"
Grep: "forbidden"
Grep: "unauthorized"
Grep: "not_allowed"
Grep: "invalid"
```

What the system rejects tells you what it considers unacceptable. If every handler rejects unauthenticated requests, that is a constraint even if no one wrote it down.

### 4. Configuration files

```
Glob: **/config/**
Glob: **/*.toml
Glob: **/*.yaml
Glob: **/*.yml
Glob: **/.env.example
```

Look for: timeout values, max retries, pool sizes, rate limits, size limits, TTLs. These are often constraints that were decided once and never revisited.

### 5. Infrastructure constraints

```
Glob: **/Dockerfile
Glob: **/docker-compose*
Glob: **/.github/workflows/**
Glob: **/terraform/**
Glob: **/k8s/**
```

Infrastructure imposes constraints: memory limits, CPU limits, timeout limits, deployment strategies. These constrain what the application code can do.

### 6. Database schema constraints

```
Grep: "NOT NULL"
Grep: "UNIQUE"
Grep: "CHECK"
Grep: "FOREIGN KEY"
Grep: "constraint"
```

Schema-level constraints are intentional data integrity rules.

### 7. External sources

**Issue tracker**: Search for tickets about outages, performance incidents, security findings. Post-incident tickets often result in new constraints.

**Monitoring**: Alert configurations reveal operational constraints (thresholds that trigger pages).

## Debt signals for constraints

Classify as **debt** when:

- A limit exists but is clearly a workaround: `// TODO: remove this limit when we fix the memory leak`
- The constraint is weaker than it should be: MD5 for password hashing, HTTP without TLS, disabled CSRF protection
- The limit contradicts a stated intent (e.g., an intent says "support uploads up to 1GB" but the config says `max_upload: 10MB`)
- Magic numbers with no explanation and no clear external driver

Classify as **ambiguous** when:

- A config value exists (e.g., `timeout: 30s`) but it is unclear whether it was chosen deliberately or is a framework default
- A limit matches a framework default exactly (might be intentional, might be accidental)

## Writing constraint candidates

State the limit, the scope, and the reason:

```
# From code: rate_limit(100, per=timedelta(minutes=1)) on all API routes
# From config: max_upload_size = 50MB
# From Dockerfile: memory limit 512MB

# Candidate:
name: "API rate limiting"
content: "All API endpoints are rate-limited to 100 requests per minute
per authenticated user. This exists to prevent abuse and to stay within
the database connection pool limit of 20 connections."
classification: truth
confidence: high
reasoning: "Applied uniformly via middleware. The connection pool
limit is documented in config comments."
```
