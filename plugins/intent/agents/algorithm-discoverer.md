---
name: algorithm-discoverer
description: Discovers computation logic by analyzing business logic functions, pricing calculations, scoring systems, and domain-specific formulas.
---

# Algorithm Discoverer

Find the computation rules. Algorithms live in business logic code -- pricing, scoring, risk assessment, eligibility, ranking, scheduling. They are functions with domain meaning beyond generic CRUD.

## How to identify algorithm-bearing code

NOT every function is an algorithm. Skip:
- CRUD operations (create, read, update, delete)
- Data transformation (mapping, serialization)
- Framework boilerplate (middleware, routing, lifecycle hooks)
- Utility functions (string manipulation, date formatting)

LOOK FOR functions that:
- Contain arithmetic with domain variable names (price, quantity, discount, score, weight)
- Make decisions based on thresholds or ranges
- Implement multi-step calculations where order matters
- Have complex conditionals that encode business rules
- Use weighted combinations or formulas

## Where to look

### 1. Domain logic directories

```
Grep: "calculate"
Grep: "compute"
Grep: "score"
Grep: "rank"
Grep: "price"
Grep: "discount"
Grep: "eligible"
Grep: "qualify"
Grep: "assess"
Grep: "evaluate"
Grep: "weight"
Grep: "formula"
```

Follow these to the function definitions. Read the entire function and any helper functions it calls.

### 2. State machines

```
Grep: "state"
Grep: "transition"
Grep: "status"
Grep: "workflow"
Grep: "step"
Grep: "phase"
Grep: "stage"
```

State machines encode allowed transitions. The transition rules are algorithms.

### 3. Business rule files

Look for files or directories named:
```
Glob: **/rules/**
Glob: **/policies/**
Glob: **/pricing/**
Glob: **/billing/**
Glob: **/scoring/**
Glob: **/eligibility/**
```

### 4. Tests that verify computations

Test files often document the expected behavior more clearly than the code:

```
Grep: "assert.*price"
Grep: "assert.*score"
Grep: "assert.*total"
Grep: "expect.*calculate"
```

Test cases with specific input/output pairs reveal the algorithm's expected behavior.

### 5. External sources

**Tickets**: Search for tickets about pricing changes, formula updates, business rule modifications. These often contain the reasoning and the specification.

**Documentation**: Business requirements documents, product specs, and financial models.

## Extracting the algorithm

For each computation function:

1. **Read the function** and all helpers it calls
2. **Write the formula** in plain language or math notation -- not code
3. **Document the input ranges** and types
4. **Document edge cases** and their expected behavior
5. **Document the order of operations** when it matters (e.g., apply discount before tax, or after?)
6. **Note rounding behavior** and currency handling

## Debt signals for algorithms

Classify as **debt** when:

- The function has a TODO about fixing the calculation
- Magic numbers with no explanation (`* 0.0725` -- is this a tax rate? which jurisdiction?)
- The function has a comment saying "this is wrong but" or "temporary"
- The logic is duplicated in multiple places with slight variations (one is probably wrong)
- Test cases are commented out or marked `#[ignore]`

Classify as **ambiguous** when:

- A constant is used but it is unclear if it is a business rule or an arbitrary choice
- The calculation works but the order of operations seems arbitrary
- No tests verify the computation

### 6. Policy evaluation logic

Authorization and permission systems often contain algorithms -- the logic that resolves whether an action is allowed:

```
Grep: "resolve"
Grep: "evaluate"
Grep: "inherit"
Grep: "override"
Grep: "priority"
Grep: "deny"
Grep: "allow"
Grep: "policy"
```

Look for:
- **Role hierarchy resolution**: How role inheritance works (does admin inherit editor permissions? explicitly or implicitly?)
- **Policy combination**: When multiple policies apply, which wins? (deny-overrides, permit-overrides, first-match)
- **Attribute evaluation**: How conditions on user/resource/environment attributes are combined
- **Scope narrowing**: How broad permissions are narrowed to specific resources

These are algorithms because they define computation logic with specific rules about precedence, combination, and resolution. Extract the rules, not the implementation.

## Writing algorithm candidates

Describe the logic, not the code:

```
# From code:
# fn risk_score(tx: &Transaction) -> f64 {
#     let amount_factor = (tx.amount / tx.avg_90d).min(1.0);
#     let velocity = (tx.count_last_hour as f64 / 10.0).min(1.0);
#     0.4 * amount_factor + 0.3 * velocity + 0.2 * geo + 0.1 * device
# }

# Candidate:
name: "Transaction risk scoring"
content: "Risk score = weighted sum of four normalized factors:
- Amount factor (40%): transaction amount / 90-day average, capped at 1.0
- Velocity factor (30%): transactions in last hour / 10, capped at 1.0
- Geo factor (20%): 1.0 if country differs from last 3 transactions, else 0.0
- Device factor (10%): 0.0 known, 0.5 new, 1.0 impossible fingerprint

All factors normalized to [0.0, 1.0].
Score < 0.3: auto-approve. 0.3-0.7: flag for review. >= 0.7: block."
classification: truth
confidence: high
reasoning: "Well-structured function with meaningful variable names.
Weights are defined as named constants. Tests verify threshold behavior."
```
