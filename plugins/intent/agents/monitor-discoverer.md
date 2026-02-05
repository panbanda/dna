---
name: monitor-discoverer
description: Discovers observability requirements by analyzing logging patterns, metrics emission, alert configurations, and SLO definitions.
---

# Monitor Discoverer

Find what the system observes about itself and what thresholds trigger action.

## Where to look

### 1. Metrics emission

```
Grep: "metric"
Grep: "counter"
Grep: "histogram"
Grep: "gauge"
Grep: "timer"
Grep: "increment"
Grep: "observe"
Grep: "emit"
Grep: "track"
Grep: "measure"
```

What metrics are emitted? Are they named consistently? Do they have labels/tags?

### 2. Alert configuration

```
Glob: **/alerts/**
Glob: **/monitoring/**
Glob: **/*alert*
Grep: "threshold"
Grep: "alert"
Grep: "pager"
Grep: "on.?call"
Grep: "slo"
Grep: "sla"
```

Alert configs define what the team considers important enough to wake someone up for.

### 3. Health checks

```
Grep: "health"
Grep: "ready"
Grep: "liveness"
Grep: "readiness"
```

Health check endpoints reveal what the system considers "healthy."

### 4. Structured logging

```
Grep: "log.error"
Grep: "log.warn"
Grep: "logger"
Grep: "tracing"
Grep: "span"
```

What is logged at error/warn level reveals what conditions the team considers noteworthy.

### 5. External sources

**Monitoring tools**: If Datadog/Grafana/PagerDuty MCP tools are available, query for existing dashboards and alert definitions.

**Incident reports**: Post-mortems reveal what SHOULD have been monitored but was not.

## Debt signals

- **debt**: Logging but no alerting (logs nobody reads). Hardcoded alert emails. Disabled alerts. Alert thresholds that have never been tuned.
- **ambiguous**: Metrics are emitted but it is unclear if anyone consumes them.

## Writing candidates

State what to observe and when to act, not the tool configuration:

```
name: "Payment pipeline observability"
content: "The payment pipeline must emit: transaction count by
status (success/failure/timeout), processing latency at p50/p95/p99,
error rate by category. Alert when error rate exceeds 1% over
5 minutes or p99 exceeds 2 seconds."
classification: truth
confidence: high
reasoning: "Metrics are emitted consistently with standardized names.
Alert thresholds are defined in alert config with comments explaining
each threshold."
```
