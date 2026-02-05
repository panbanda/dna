---
name: evaluation-discoverer
description: Discovers success criteria by analyzing test suites, assertion patterns, CI configuration, and acceptance criteria in tickets.
---

# Evaluation Discoverer

Find what defines "correct." Evaluations are encoded in tests, CI pipelines, acceptance criteria, and monitoring thresholds. The goal is to extract the CRITERIA, not the test code.

## Where to look

### 1. Test descriptions and names

```text
Glob: **/tests/**
Glob: **/test/**
Glob: **/*_test.*
Glob: **/*.test.*
Glob: **/*.spec.*
Glob: **/*_spec.*
Glob: **/__tests__/**
```

Read test names first. Good test names ARE evaluations:
- `test_expired_session_redirects_to_login` -- that is an evaluation
- `test_negative_balance_rejected` -- that is an invariant
- `test_concurrent_updates_do_not_lose_data` -- that is a critical evaluation

For each test file, extract the behaviors being verified, not the test mechanics.

### 2. Property-based tests

```text
Grep: "proptest"
Grep: "quickcheck"
Grep: "hypothesis"
Grep: "property"
Grep: "forall"
Grep: "arbitrary"
```

Property tests encode invariants -- things that must ALWAYS be true regardless of input. These are high-value evaluations.

### 3. Integration and E2E tests

```text
Glob: **/e2e/**
Glob: **/integration/**
Glob: **/acceptance/**
```

These test system-level behavior and often correspond directly to evaluation artifacts.

### 4. CI configuration

```text
Glob: **/.github/workflows/**
Glob: **/.gitlab-ci.*
Glob: **/Jenkinsfile
Glob: **/.circleci/**
```

What the CI pipeline checks reveals what the team considers "correct":
- Which tests must pass before merge?
- What coverage thresholds exist?
- What linting or security scanning is enforced?
- Are there performance benchmarks that must pass?

### 5. Assertion patterns

```text
Grep: "assert"
Grep: "expect"
Grep: "should"
Grep: "must"
Grep: "verify"
```

Focus on assertions in integration tests and E2E tests. Unit test assertions are usually too granular for evaluation artifacts. Look for assertions that verify business rules, not implementation details.

### 6. External sources

**Tickets**: Acceptance criteria in tickets are direct evaluations. Search for tickets with "acceptance criteria", "definition of done", "success criteria".

**QA documentation**: Test plans, test matrices, regression test lists.

## Grouping evaluations

Group related test assertions into a single evaluation artifact. Do not create one artifact per test case. Instead:

```yaml
# From 5 separate tests:
# - test_valid_login_returns_token
# - test_invalid_password_returns_generic_error
# - test_expired_session_redirects
# - test_max_5_concurrent_sessions
# - test_6th_session_invalidates_oldest

# One evaluation candidate:
name: "Authentication correctness"
content: "Authentication is correct when:
1. Valid credentials produce a session token within 500ms
2. Invalid credentials return a generic error (no user enumeration)
3. Expired sessions redirect to login without data loss
4. Concurrent sessions permitted up to 5 per user
5. Session 6 invalidates the oldest"
```

## Debt signals for evaluations

Classify as **debt** when:

- Tests are marked `#[ignore]`, `@skip`, `xit`, or `xdescribe`
- Tests contain `// TODO: fix this test`
- Assertions are commented out
- Test expectations are obviously wrong ("assert balance == -50" when balance should never be negative)
- Tests mock so much that they test nothing real

Classify as **ambiguous** when:

- Tests exist but it is unclear if the asserted behavior is intentional or just what happens to work
- Tests use hardcoded values without explanation (is `assert_eq!(result, 42)` a business rule or an arbitrary test value?)
- No tests exist for a critical area (absence is a signal, but it could mean the area is trivial or it could mean it is untested)

## Writing evaluation candidates

State the criteria, not the test code:

```yaml
# Candidate:
name: "Non-negative account balance"
content: "Account balance must never go negative. All withdrawal
operations must check available balance atomically and reject with
insufficient-funds if the withdrawal would result in a negative
balance. This must hold under concurrent requests."
classification: truth
confidence: high
reasoning: "Property-based test verifies this invariant under
concurrent load. Three regression tests reference past incidents
where this was violated."
```
