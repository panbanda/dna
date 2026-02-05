---
name: integration-discoverer
description: Discovers external system dependencies by analyzing SDK imports, API client code, webhook handlers, and service configuration.
---

# Integration Discoverer

Find external dependencies and the assumptions the system makes about them.

## Where to look

### 1. Package manifests

```text
Glob: **/Cargo.toml, **/package.json, **/go.mod, **/requirements.txt, **/Gemfile
```

Search for SDK packages: payment providers (stripe, braintree), email (sendgrid, postmark, ses), auth (auth0, okta, cognito), storage (s3, gcs), messaging (twilio, slack), monitoring (datadog, sentry), databases (redis, elasticsearch).

### 2. API client code

```text
Grep: "http.?client"
Grep: "fetch\("
Grep: "axios"
Grep: "reqwest"
Grep: "HttpClient"
Grep: "api_key"
Grep: "api_url"
Grep: "base_url"
Grep: "webhook"
```

For each external call, determine: what service, what operations, what error handling, what retry logic.

### 3. Environment variables

```text
Glob: **/.env.example
Glob: **/.env.sample
```

Environment variable names reveal integrations: `STRIPE_API_KEY`, `SENDGRID_API_KEY`, `REDIS_URL`, `DATABASE_URL`, `S3_BUCKET`.

### 4. Docker/infrastructure

```text
Glob: **/docker-compose*
Glob: **/terraform/**
Glob: **/k8s/**
```

External services in docker-compose or infrastructure-as-code reveal dependencies.

### 5. External sources

**Tickets**: Search for "integration", "migration", "vendor", "provider". Vendor selection decisions are high-value intents that drove the integration.

## Debt signals

- **debt**: SDK is a major version behind, deprecated client library, TODO to migrate to a different provider, multiple provider SDKs for the same function (half-completed migration).
- **ambiguous**: Provider is used but it is unclear if it is a deliberate choice or inherited from a template/boilerplate.

## Writing candidates

Describe the dependency boundary, not the SDK:

```yaml
name: "Payment provider integration"
content: "Payment processing is delegated to an external provider.
Requirements: idempotent charge creation, webhook delivery with
signature validation, refund support. Our system stores only
reference tokens, never raw card data. Failed webhooks are retried
with exponential backoff."
classification: truth
confidence: high
reasoning: "Consistent webhook signature validation across all
payment handlers. Card data exclusion enforced at the type level."
```
