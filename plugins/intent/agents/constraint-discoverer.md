---
name: constraint-discoverer
description: Discovers hard limits by analyzing validation logic, error handling, security middleware, configuration bounds, and infrastructure constraints.
---

# Constraint Discoverer

Find the hard boundaries. Constraints are encoded in validation functions, config limits, middleware, error handlers, and infrastructure.

The critical distinction: a constraint is a limit that SHOULD exist. A workaround or hack that imposes a limit is debt, not a constraint.

## Where to look

### 1. Input validation

```text
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

```text
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

```text
Grep: "reject"
Grep: "deny"
Grep: "forbidden"
Grep: "unauthorized"
Grep: "not_allowed"
Grep: "invalid"
```

What the system rejects tells you what it considers unacceptable. If every handler rejects unauthenticated requests, that is a constraint even if no one wrote it down.

### 4. Configuration files

```text
Glob: **/config/**
Glob: **/*.toml
Glob: **/*.yaml
Glob: **/*.yml
Glob: **/.env.example
```

Look for: timeout values, max retries, pool sizes, rate limits, size limits, TTLs. These are often constraints that were decided once and never revisited.

### 5. Infrastructure constraints

```text
Glob: **/Dockerfile
Glob: **/docker-compose*
Glob: **/.github/workflows/**
Glob: **/terraform/**
Glob: **/k8s/**
```

Infrastructure imposes constraints: memory limits, CPU limits, timeout limits, deployment strategies. These constrain what the application code can do.

### 6. Authorization and permission model

Authorization is a cross-cutting constraint system. Discover the model, not just individual checks.

#### Identify the model type

```text
Grep: "role"
Grep: "permission"
Grep: "policy"
Grep: "scope"
Grep: "claim"
Grep: "grant"
Grep: "capability"
Grep: "privilege"
Grep: "entitlement"
Grep: "tenant"
Grep: "org"
Grep: "team"
Grep: "owner"
Grep: "member"
```

Determine which model the system uses:

| Pattern | Model | Signals |
|---------|-------|---------|
| Users have roles, roles have permissions | RBAC | `role`, `has_role`, `role_permissions`, role hierarchy tables |
| Decisions based on user/resource/env attributes | ABAC | `attribute`, `condition`, `policy_engine`, attribute matchers |
| Relationships between users and objects | ReBAC / FGA | `relation`, `tuple`, `check`, `expand`, object-relation-user triples |
| Explicit allow/deny lists per resource | ACL | `access_list`, `acl`, `allowed_users`, per-resource permission entries |
| Bearer tokens with scoped capabilities | Capability / OAuth scopes | `scope`, `bearer`, `token.scopes`, `required_scopes` |

Many systems combine models (e.g., RBAC for coarse access + ABAC for row-level filtering).

#### Find the permission boundaries

```text
Grep: "admin"
Grep: "super.?admin"
Grep: "sudo"
Grep: "bypass"
Grep: "override"
Grep: "escalat"
Grep: "impersonat"
```

Look for operations that are restricted even for administrators. These are the hardest constraints in the system:
- "Even admins cannot view raw payment card data"
- "Impersonation requires audit log entry and expires after 1 hour"
- "No role can bypass the deletion cooldown period"

#### Find tenant isolation

```text
Grep: "tenant"
Grep: "org.?id"
Grep: "organization"
Grep: "workspace"
Grep: "team.?id"
Grep: "WHERE.*tenant"
Grep: "row.?level"
Grep: "RLS"
```

Multi-tenancy constraints are critical. Determine:
- Where is tenant isolation enforced? (middleware, database, both?)
- Can any operation cross tenant boundaries?
- How are shared/global resources handled?

#### Find permission check patterns

```text
Grep: "authorize"
Grep: "has_permission"
Grep: "check_access"
Grep: "guard"
Grep: "protect"
Grep: "require_permission"
Grep: "permission"
Grep: "allowed"
```

Adapt based on the language and framework detected in recon. Search for the idiomatic authorization check patterns used by whatever framework is in play.

Look for WHERE permission checks happen:
- Middleware/decorator level (uniform enforcement)
- Inside handler logic (case-by-case, harder to audit)
- Database query level (row-level security)
- Frontend only (debt -- backend must enforce)

#### Look for FGA/Zanzibar patterns

```text
Grep: "openfga"
Grep: "zanzibar"
Grep: "spicedb"
Grep: "ory.?keto"
Grep: "cerbos"
Grep: "cedar"
Grep: "oso"
Grep: "casbin"
Grep: "rego"
Grep: "authz"
Glob: **/*.polar
Glob: **/*.cedar
Glob: **/*.rego
Glob: **/policy/**
Glob: **/policies/**
Glob: **/authorization/**
Glob: **/permissions/**
```

If an FGA or policy engine is used, the authorization model and policy definitions are high-value artifacts. Extract:
- The type/relation definitions (the authorization model schema)
- Key policy rules and their intent
- The relationship between the policy model and the domain model

#### Write authorization constraint candidates

Authorization constraints should capture the MODEL and the BOUNDARIES, not individual permission checks:

```yaml
# Candidate:
name: "Tenant data isolation"
content: "Every database query must be scoped to the requesting user's
tenant. Tenant isolation is enforced at the database level via row-level
security policies, not in application code. No API endpoint may return
data belonging to a different tenant. Cross-tenant operations are
impossible by design -- there is no admin bypass for tenant boundaries."
classification: truth
confidence: high
reasoning: "RLS policies on all tables. Middleware injects tenant_id.
Integration tests verify cross-tenant access returns empty results."
```

```yaml
# Candidate:
name: "Permission model"
content: "Authorization uses role-based access control with four roles:
viewer (read-only), editor (read-write own resources), admin
(read-write all resources in org), owner (admin + billing + member
management). Role hierarchy: owner > admin > editor > viewer.
Permissions are checked at the API middleware layer, never in
frontend only."
classification: truth
confidence: high
reasoning: "Role enum with four variants. Middleware checks on all
routes. Tests verify each role's boundaries."
```

### 7. Database schema constraints

```text
Grep: "NOT NULL"
Grep: "UNIQUE"
Grep: "CHECK"
Grep: "FOREIGN KEY"
Grep: "constraint"
```

Schema-level constraints are intentional data integrity rules.

### 8. External sources

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

```yaml
# From code: rate_limit(100, per=timedelta(minutes=1)) on all API routes
# From config: max_upload_size = 50MB
# From Dockerfile: memory limit 512MB

# Candidate:
name: "API rate limiting"
content: "All API endpoints are rate-limited to 100 requests per minute
per authenticated user. This prevents abuse and protects the database
connection pool from exhaustion."
context: "API throttling, abuse prevention, connection pool protection,
per-user limits -- added after March 2023 scraping incident that
exhausted the connection pool"
classification: truth
confidence: high
reasoning: "Applied uniformly via middleware. The connection pool
limit is documented in config comments."
```
