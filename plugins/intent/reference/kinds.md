# Intent Template: Kind Reference

The intent template provides 11 kinds for truth-driven governance. This reference covers the 6 kinds not covered by dedicated skills (pace, monitor, glossary, integration, reporting, compliance). For intent, contract, constraint, algorithm, and evaluation, see the corresponding skills.

## pace

**What it captures:** Change velocity for a specific area. How fast or slow changes should move through review and deployment.

**Label convention:** None required, but `--label domain=<domain>` is useful. Label keys must be registered via `dna label add` before use. Run `dna label list` to see available keys.

**Examples:**

```bash
dna add pace \
  "Auth model: slow. Changes to authentication, authorization, or session management require security review and staged rollout. No same-day deploys." \
  --name "Auth change pace" \
  --label domain=auth

dna add pace \
  "Marketing pages: fast. Content changes can deploy immediately without review. Layout changes need design sign-off but no engineering review." \
  --name "Marketing page pace" \
  --label domain=marketing
```

**When to use:** When different parts of the system should move at different speeds. Prevents a junior developer from shipping auth changes with the same process as a typo fix.

---

## monitor

**What it captures:** An operational metric, SLO, or alert condition. What must be observed and what thresholds matter.

**Label convention:** `--label domain=<domain>`

**Examples:**

```bash
dna add monitor \
  "Payment processing pipeline: emit transaction count by status (success/failure/timeout), processing latency at p50/p95/p99, error rate by category. Alert when error rate exceeds 1% over 5 minutes or p99 exceeds 2 seconds." \
  --name "Payment pipeline observability" \
  --label domain=payments

dna add monitor \
  "p99 API latency < 200ms for customer-facing endpoints. Measured at the load balancer, not the application. Alert on sustained violations (3+ minutes)." \
  --name "API latency SLO" \
  --label domain=api
```

**When to use:** When the absence of monitoring has caused incidents, or when specific metrics are contractually required (SLAs).

---

## glossary

**What it captures:** A domain term with precise meaning. Prevents ambiguity where different people use the same word to mean different things.

**Label convention:** None required.

**Examples:**

```bash
dna add glossary \
  "Member: an authenticated individual who has completed onboarding and has an active subscription. Has a profile, can create content. Distinct from 'visitor' (unauthenticated). The term 'user' is ambiguous -- use 'member' or 'visitor' instead." \
  --name "Member" \
  --label domain=identity

dna add glossary \
  "ICP (Ideal Customer Profile): B2B SaaS company, 50-500 employees, Series A or later, using a modern tech stack. This is the sales team's targeting definition, not a technical filter." \
  --name "ICP" \
  --label domain=sales
```

**When to use:** When a term has caused confusion between teams, when onboarding new engineers, or when a term has a domain-specific meaning that differs from common usage.

---

## integration

**What it captures:** How an external system connects. The provider, the protocol, and the boundaries of the dependency.

**Label convention:** `--label provider=<provider-name>`

**Examples:**

```bash
dna add integration \
  "Payment processing is delegated to an external provider. The provider must support idempotent charge creation. Our system stores only a reference token, never raw card data. Webhook delivery must be verified via signature validation. Failed webhooks must be retried with exponential backoff." \
  --name "Payment provider integration" \
  --label provider=stripe \
  --label domain=billing

dna add integration \
  "Email delivery via transactional email provider. API key authenticated. Must support: send, batch send, template rendering. We do not rely on delivery status webhooks for critical flows -- all confirmation is based on our send-side record." \
  --name "Email provider integration" \
  --label provider=sendgrid \
  --label domain=notifications
```

**When to use:** When the system depends on an external service and you need to document the boundary, the assumptions, and what happens when the provider is unavailable.

---

## reporting

**What it captures:** A business or compliance query that must be answerable from the data. What must be reportable.

**Label convention:** `--label audience=<who-needs-it>`

**Examples:**

```bash
dna add reporting \
  "Monthly business review requires: revenue by product line, customer acquisition cost by channel, churn rate with cohort breakdown, and runway projection. All figures must reconcile with the accounting system to within $1." \
  --name "Monthly business review data" \
  --label audience=executive \
  --label domain=finance

dna add reporting \
  "Support team needs real-time visibility into: open ticket count by priority, average first-response time (target: < 4 hours for P1), and customer satisfaction score per agent." \
  --name "Support team metrics" \
  --label audience=support \
  --label domain=support
```

**When to use:** When someone needs to answer a question from the data and the system must be designed to support that query. Prevents "we can't report on X because we never stored the data."

---

## compliance

**What it captures:** A regulatory or legal obligation with its source. Not general best practices -- specific requirements from a named regulation.

**Label convention:** `--label regulation=<regulation-name>` (gdpr, hipaa, pci-dss, soc2, ccpa, etc.)

**Examples:**

```bash
dna add compliance \
  "User data deletion requests must be fulfilled within 72 hours. Deletion must propagate to: primary database, search indices, analytics pipelines, backup systems (marked for exclusion from restore), and third-party integrations. A deletion receipt must be generated listing all systems purged." \
  --name "GDPR right to erasure" \
  --label regulation=gdpr \
  --label domain=data

dna add compliance \
  "Credit card numbers must never be stored in our systems. All card processing happens at the payment provider. PAN data must not appear in logs, error messages, or analytics. Truncated last-4 digits may be stored for display purposes only." \
  --name "PCI-DSS cardholder data" \
  --label regulation=pci-dss \
  --label domain=billing
```

**When to use:** When a legal or regulatory requirement applies. Always cite the regulation. General security practices belong in constraints, not compliance.
