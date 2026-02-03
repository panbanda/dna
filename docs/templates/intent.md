# Intent Template

Truth-driven governance for system identity based on the [intent-starter](https://github.com/billeisenhauer/intent-starter) pattern.

## Philosophy

Software identity should be governed, not inferred. Rather than reconstructing meaning from code, this template makes intent explicit and enforceable. The core principle: **code is temporary; identity is binding**.

Systems fail not from wrong code, but from *meaning drift* - when implementations change faster than intentions are preserved.

## When to Use

Use this template for:
- Production software systems
- APIs and services
- Monoliths and microservices
- Any codebase where you need to preserve system identity across changes

## Kinds

### intent

**Purpose**: Declarative 'must' statements defining what the system is.

**What belongs here**:
- User-observable outcomes
- Business rules
- Legitimacy constraints
- Absolute language ("must", "must not")

**What doesn't belong**:
- Implementation details
- Framework references
- Procedures or how-to
- Speculation

**Labels**: None standard

**Examples**:
```bash
dna add intent "Orders must not ship until payment is confirmed"
dna add intent "Users must verify email before posting"
dna add intent "Deleted data must be unrecoverable after 30 days"
dna add intent "Sessions must expire after 30 minutes of inactivity"
```

### contract

**Purpose**: External promises others depend on.

**What belongs here**:
- API endpoint behaviors
- Event schemas
- Interface signatures
- Error codes and responses

**What doesn't belong**:
- Internal APIs
- Unstable interfaces
- Implementation shortcuts

**Labels**: `--label version=v1`, `--label service=orders`

**Examples**:
```bash
dna add contract "POST /orders returns 201 with order_id on success"
dna add contract "POST /orders returns 400 with error_code on validation failure"
dna add contract "OrderCreated event contains order_id, customer_id, timestamp"
dna add contract "GET /health returns 200 when service is operational"
```

### algorithm

**Purpose**: Computation rules defining system behavior.

**What belongs here**:
- Formulas
- Thresholds
- Weights
- Scoring parameters
- Rate limits

**What doesn't belong**:
- Implementation code
- Performance optimizations
- Language-specific syntax

**Labels**: `--label domain=pricing`, `--label domain=scoring`

**Examples**:
```bash
dna add algorithm "discount = base_price * 0.1 when quantity > 10"
dna add algorithm "shipping_cost = base_rate + (weight_kg * 2.50) when distance > 100km"
dna add algorithm "relevance_score = 0.7 * title_match + 0.3 * body_match"
dna add algorithm "rate_limit = 100 requests per minute per API key"
dna add algorithm "retry_delay = min(base_delay * 2^attempt, max_delay)"
```

### evaluation

**Purpose**: Executable tests that judge system identity.

Evaluations are the highest authority - they supersede code style, diffs, and subjective opinions. If an evaluation fails, the system is rejected.

**What belongs here**:
- Invariants (always-true properties)
- Scenarios (BDD-style tests)
- Regressions (past failures that must not recur)

**What doesn't belong**:
- Implementation unit tests
- Performance benchmarks
- Exploratory tests

**Labels**: `--label type=invariant|scenario|regression`

**Examples**:
```bash
# Invariants - properties that must always hold
dna add evaluation "Account balance >= 0" --label type=invariant
dna add evaluation "Every order has exactly one customer" --label type=invariant
dna add evaluation "Passwords are never stored in plaintext" --label type=invariant

# Scenarios - BDD-style feature tests
dna add evaluation "Given premium user with cart > \$50, when checkout, then shipping is free" --label type=scenario
dna add evaluation "Given expired token, when API call, then 401 returned" --label type=scenario

# Regressions - past failures that must not recur
dna add evaluation "Issue #42: cart rounding error must not recur" --label type=regression
dna add evaluation "Issue #156: duplicate orders on double-click" --label type=regression
```

### pace

**Purpose**: Change governance - what level of ceremony is required to change something.

Pace is about permission to change, not speed of work. It prevents accidental erosion of system identity.

**Layers**:
- **fast**: Regenerate freely, minimal review (UI styling, log messages, adapters)
- **medium**: Review required (workflows, orchestration, state binding)
- **slow**: Deliberate change, rare (invariants, public APIs, security model)

**What doesn't belong**:
- Sprint planning
- Velocity metrics
- Time estimates

**Labels**: `--label layer=fast|medium|slow`

**Examples**:
```bash
dna add pace "UI styling and layout: fast" --label layer=fast
dna add pace "Log message formatting: fast" --label layer=fast
dna add pace "Business workflow orchestration: medium" --label layer=medium
dna add pace "Database schema: medium" --label layer=medium
dna add pace "Authentication model: slow" --label layer=slow
dna add pace "Public API endpoints: slow" --label layer=slow
dna add pace "Core invariants: slow" --label layer=slow
```

### monitor

**Purpose**: Observable contracts - what must be measured, not how.

**What belongs here**:
- Metric definitions with formulas
- SLO targets with thresholds
- Health criteria

**What doesn't belong**:
- Dashboard layouts
- Alerting infrastructure
- Tool-specific configuration

**Labels**: `--label type=metric|slo|health`

**Examples**:
```bash
dna add monitor "p99_latency: 99th percentile response time in ms, target < 200ms" --label type=slo
dna add monitor "error_rate = 5xx_responses / total_responses, alert when > 0.1% for 5min" --label type=metric
dna add monitor "availability = successful_requests / total_requests, target >= 99.9%" --label type=slo
dna add monitor "healthy: all downstream dependencies respond within 5s" --label type=health
```

### glossary

**Purpose**: Domain term definitions with precise meaning.

Enables consistent communication between humans and AI. When someone says a term, everyone (including AI) should understand exactly what it means in this system's context.

**What belongs here**:
- Business terms
- Acronyms
- Domain-specific definitions
- ICP definitions

**Labels**: `--label domain=sales`, `--label domain=product`

**Examples**:
```bash
dna add glossary "ICP: B2B SaaS companies, 50-500 employees, Series A+, using cloud infrastructure"
dna add glossary "Active user: any user who logged in within the past 30 days"
dna add glossary "Churn: customer who cancels subscription within billing period"
dna add glossary "MRR: Monthly Recurring Revenue - sum of all active subscription amounts"
dna add glossary "Conversion: visitor who completes purchase within session"
```

### integration

**Purpose**: External system bindings - providers, APIs, SLA terms.

Changing these is a major undertaking. They represent architectural commitments.

**What belongs here**:
- Provider choices
- API dependencies
- SLA terms from contracts
- Integration constraints

**Labels**: `--label provider=stripe`, `--label contract=acme-enterprise`

**Examples**:
```bash
dna add integration "Payment provider: Stripe" --label provider=stripe
dna add integration "Stripe webhook signatures required for all events" --label provider=stripe
dna add integration "CRM: Salesforce, leads sync bidirectionally" --label provider=salesforce
dna add integration "Maps: Google Maps Platform, 100 req/s limit" --label provider=google-maps
dna add integration "Enterprise SLA: 99.9% uptime for Acme Corp" --label provider=internal --label contract=acme
```

### reporting

**Purpose**: Business or compliance information that must be queryable.

Different from monitoring (operational health) - this is about business intelligence and compliance needs.

**What belongs here**:
- Board reporting requirements
- Compliance audit needs
- Business intelligence queries

**Labels**: `--label audience=board`, `--label compliance=sox`

**Examples**:
```bash
dna add reporting "Monthly revenue by customer segment must be queryable"
dna add reporting "User acquisition by channel must be tracked"
dna add reporting "GDPR data access requests must be fulfillable within 72 hours"
dna add reporting "SOC2 audit logs must be retained for 7 years"
dna add reporting "Daily active users by geography for board reporting"
```

### compliance

**Purpose**: Regulatory or legal obligations.

These are non-negotiable requirements from external regulatory bodies.

**What belongs here**:
- GDPR requirements
- HIPAA obligations
- PCI-DSS rules
- SOC2 controls
- Industry-specific regulations

**Labels**: `--label regulation=gdpr|hipaa|pci-dss|soc2`

**Examples**:
```bash
dna add compliance "PII must be deletable within 30 days of request" --label regulation=gdpr
dna add compliance "PHI must be encrypted at rest and in transit" --label regulation=hipaa
dna add compliance "Credit card numbers must not be logged" --label regulation=pci-dss
dna add compliance "Access logs must be immutable" --label regulation=soc2
dna add compliance "User consent required before data collection" --label regulation=gdpr
```

### constraint

**Purpose**: Technical limits and boundaries.

These are capacity, performance, or architectural constraints that bound what the system can do.

**What belongs here**:
- Capacity limits
- Performance boundaries
- Architectural constraints
- Resource limits

**Labels**: `--label type=capacity|performance|architecture`

**Examples**:
```bash
dna add constraint "Maximum file upload size: 100MB" --label type=capacity
dna add constraint "API response time must be < 500ms at p95" --label type=performance
dna add constraint "Must run stateless for horizontal scaling" --label type=architecture
dna add constraint "Maximum 1000 concurrent WebSocket connections per node" --label type=capacity
dna add constraint "Database queries must not exceed 100ms" --label type=performance
```

## Usage

Initialize a project with the intent template:

```bash
dna init --template intent
```

List all available templates:

```bash
dna init --list-templates
```

## Querying

Search across all truth artifacts:

```bash
dna search "payment processing"
```

List all artifacts of a specific kind:

```bash
dna list --kind intent
dna list --kind evaluation
```

Filter by labels:

```bash
dna search "stripe" --label provider=stripe
dna list --kind compliance --label regulation=gdpr
```

## Best Practices

1. **Keep artifacts atomic**: One truth per artifact, not documents
2. **Use labels for grouping**: Related truths share labels, not a single artifact
3. **Include examples in content**: Help AI understand with concrete examples
4. **Review pace layer assignments**: Ensure critical items are marked slow
5. **Link evaluations to intents**: Evaluations verify that intents are maintained
