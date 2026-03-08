---
name: escalation-discoverer
description: Discovers decision boundaries and human-in-the-loop requirements by analyzing approval workflows, permission checks, manual overrides, and risk-gated operations.
---

# Escalation Discoverer

Find where the system requires human involvement before proceeding. Escalations are encoded in approval workflows, manual review gates, permission boundaries, confirmation dialogs, and operational runbooks.

The critical distinction: an escalation is a deliberate decision boundary where autonomous action is inappropriate. A missing feature that forces manual work is debt, not an escalation.

## Where to look (priority order)

### 1. Approval and review workflows

```text
Grep: "approv"
Grep: "review"
Grep: "sign.?off"
Grep: "authorize"
Grep: "confirm"
Grep: "manual"
Grep: "human"
Grep: "override"
Grep: "bypass"
```

Any code path that requires explicit human approval before proceeding encodes an escalation. Look for:
- Multi-step approval chains
- Manager or admin approval gates
- Review queues and pending states
- Confirmation dialogs for destructive actions

### 2. Threshold-based gates

```text
Grep: "threshold"
Grep: "limit"
Grep: "exceeds"
Grep: "above"
Grep: "below"
Grep: "maximum"
Grep: "minimum"
Grep: "too.?many"
Grep: "too.?large"
```

Thresholds that trigger different behavior often encode escalation boundaries. Look for:
- Amount limits that require approval (e.g., refunds over $X)
- Quantity limits that trigger review (e.g., bulk operations over N items)
- Risk scores that gate automated processing

### 3. Permission escalation patterns

```text
Grep: "escalat"
Grep: "elevat"
Grep: "sudo"
Grep: "admin.?only"
Grep: "requires.?admin"
Grep: "privileged"
Grep: "super.?user"
Grep: "impersonat"
```

Operations restricted to higher permission levels reveal decision boundaries. If only admins can do something, there is an implicit escalation: "non-admin agents must escalate this action."

### 4. Operational runbooks and playbooks

```text
Glob: **/runbook*
Glob: **/playbook*
Glob: **/docs/ops/**
Glob: **/docs/operations/**
Glob: **/docs/procedures/**
Grep: "on-call"
Grep: "pager"
Grep: "incident"
Grep: "page"
```

Runbooks explicitly document when to escalate and to whom. They are the purest source of escalation artifacts.

### 5. Feature flags and rollout controls

```text
Grep: "feature.?flag"
Grep: "rollout"
Grep: "canary"
Grep: "percentage"
Grep: "gradual"
Grep: "kill.?switch"
Grep: "circuit.?breaker"
```

Gradual rollout mechanisms encode a decision: "Do not go fully autonomous with this change. A human must monitor and decide when to proceed."

### 6. Destructive operation safeguards

```text
Grep: "delete"
Grep: "destroy"
Grep: "drop"
Grep: "purge"
Grep: "wipe"
Grep: "reset"
Grep: "irreversible"
Grep: "permanent"
Grep: "cannot.?undo"
Grep: "soft.?delete"
Grep: "hard.?delete"
Grep: "grace.?period"
Grep: "cooldown"
```

Systems that distinguish between soft and hard delete, or that impose cooldown periods, encode an escalation: "Do not permanently destroy without human confirmation."

### 7. Financial and compliance gates

```text
Grep: "refund"
Grep: "credit"
Grep: "charge"
Grep: "invoice"
Grep: "payout"
Grep: "transfer"
Grep: "audit"
Grep: "compliance"
Grep: "legal"
```

Financial operations often have approval thresholds. Look for dollar amounts or percentage-based limits that trigger different handling.

### 8. External sources

**Issue tracker**: Search for tickets about approval workflows, permission escalation, and operational procedures.

**Runbooks**: If available via MCP, runbooks are the highest-value source for escalation discovery.

**Incident reports**: Post-incident actions often create new escalation requirements: "Going forward, any change to X requires approval from Y."

## Debt signals for escalations

Classify as **debt**, not truth, when:

- A manual step exists only because automation was never built ("We manually approve every order" when there is no risk-based reason for it)
- The escalation threshold is clearly wrong ("Require manager approval for refunds over $1" -- the threshold is too low to be intentional)
- The escalation references a role or team that no longer exists

Classify as **ambiguous** when:

- A permission check exists but it is unclear whether it is a security requirement or a workflow gate
- A confirmation dialog exists but it is unclear whether it protects against user error or represents a genuine decision boundary

## Writing escalation candidates

State the trigger, the threshold, who decides, and why:

```yaml
# From code: if refund_amount > 500 { require_manager_approval() }
# From runbook: "Refunds over $500 require manager sign-off per
# finance policy FIN-042"

# Candidate:
name: "Refund approval threshold"
content: "Refunds exceeding $500 require approval from a manager
or finance team member before processing. The approver must see:
the original order, the refund reason, the customer history, and
whether the refund is full or partial. Autonomous refund processing
below this threshold is acceptable. This exists because uncontrolled
large refunds were identified as a fraud vector in the 2023
finance audit."
context: "refund processing, financial controls, fraud prevention,
approval workflow, finance policy FIN-042 -- introduced after 2023
audit finding about unauthorized large refunds"
classification: truth
confidence: high
reasoning: "Explicit threshold in code matches documented finance
policy. Manager approval workflow is tested in integration tests."
```
