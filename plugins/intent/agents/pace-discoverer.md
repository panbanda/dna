---
name: pace-discoverer
description: Discovers change velocity by analyzing git history, CI configuration, code review policies, and deployment gates.
---

# Pace Discoverer

Find how fast different parts of the system are allowed to change. Pace is encoded in git history (how often areas change), CI config (what gates exist), and team policies (review requirements).

## Where to look

### 1. Git history by directory

```bash
# Find most frequently changed directories (last 6 months)
git log --since="6 months ago" --name-only --pretty=format: | sort | uniq -c | sort -rn | head -50

# Find directories that rarely change
git log --since="1 year ago" --name-only --pretty=format: | sort -u > changed_files.txt
```

High-churn areas are likely "fast" pace. Low-churn areas may be "slow" pace (critical infrastructure, security, billing) or dead code.

### 2. Code review configuration

```
Glob: **/.github/CODEOWNERS
Glob: **/CODEOWNERS
Glob: **/.github/workflows/ci.yml
```

CODEOWNERS reveals which areas require specific reviewers. Areas with more required reviewers have slower pace.

### 3. Branch protection

```bash
# If GitHub CLI available:
gh api repos/{owner}/{repo}/branches/main/protection
```

Protected branches with required reviews, status checks, and approvals indicate controlled pace.

### 4. Deployment configuration

```
Glob: **/.github/workflows/deploy*
Glob: **/deploy/**
Grep: "canary"
Grep: "staged"
Grep: "rollout"
Grep: "feature.?flag"
```

Canary deployments, staged rollouts, and feature flags indicate areas where the team controls change velocity.

### 5. External sources

**Tickets**: Search for "freeze", "moratorium", "stabilize", "slow down". These indicate pace decisions.

**Documentation**: Deployment runbooks, change management policies.

## Debt signals

- **debt**: Area that should change slowly (auth, billing) but has no additional review gates. Area marked as "frozen" but still receiving commits.
- **ambiguous**: Area that rarely changes -- is it stable-by-design or just abandoned?

## Writing candidates

```
name: "Auth change pace"
content: "Authentication and authorization: slow. Changes require
security review and staged rollout. No same-day deploys for auth
changes."
classification: truth
confidence: medium
reasoning: "CODEOWNERS requires security team review for auth/.
CI has additional security scan step for changes to auth/. However,
no explicit policy document found."
```
