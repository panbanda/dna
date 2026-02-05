---
name: reporting-discoverer
description: Discovers business reporting requirements by analyzing query patterns, analytics events, dashboard configurations, and data models designed for aggregation.
---

# Reporting Discoverer

Find what must be queryable. Reporting requirements are encoded in database views, analytics events, aggregation queries, and dashboard configurations.

## Where to look

### 1. Analytics and tracking events

```
Grep: "analytics"
Grep: "track"
Grep: "event"
Grep: "segment"
Grep: "amplitude"
Grep: "mixpanel"
Grep: "gtag"
Grep: "posthog"
```

What events are tracked reveals what the business considers important to measure.

### 2. Database views and materialized queries

```
Grep: "CREATE VIEW"
Grep: "CREATE MATERIALIZED"
Grep: "materialized"
```

Database views exist because someone needed a specific aggregation. The view definition reveals the reporting requirement.

### 3. Report generation code

```
Glob: **/reports/**
Glob: **/reporting/**
Grep: "report"
Grep: "export"
Grep: "csv"
Grep: "dashboard"
```

### 4. Data models with aggregation fields

Look for tables or types with fields like: `total_count`, `sum_amount`, `last_activity`, `metrics`, `stats`. These exist to support reporting.

### 5. External sources

**Dashboard tools**: If Metabase, Looker, or Grafana MCP tools are available, query for existing dashboards. Each dashboard represents a reporting requirement.

**Tickets**: Search for "report", "dashboard", "metrics", "KPI", "analytics".

## Debt signals

- **debt**: Analytics events that are tracked but never queried (dead tracking). Reports that pull from raw tables instead of views (fragile). Hardcoded date ranges in report queries.
- **ambiguous**: Analytics events exist but it is unclear if the business still cares about them.

## Writing candidates

```
name: "Monthly revenue by product line"
content: "Monthly business review requires revenue broken down by
product line. Figures must reconcile with the accounting system
to within $1. Must be reproducible from raw transaction data."
classification: truth
confidence: medium
reasoning: "Database view 'monthly_revenue_by_product' exists and
is queried by the reporting service. However, no documentation
confirms this is a standing requirement vs. a one-time report."
```
