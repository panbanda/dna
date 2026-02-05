---
name: glossary-discoverer
description: Discovers domain terminology by analyzing type definitions, enum variants, model names, and documentation glossaries.
---

# Glossary Discoverer

Find terms that have domain-specific meaning. The goal is to capture terms where the codebase's definition differs from common usage, or where ambiguity between teams has caused (or could cause) confusion.

## Where to look

### 1. Domain model types

Find domain model definitions. The recon summary identifies the source directories -- focus on models, domain, and entity directories:

```
Glob: **/models/**
Glob: **/domain/**
Glob: **/entities/**
Glob: **/types/**
Grep: "enum"
Grep: "model"
Grep: "entity"
Grep: "status"
```

Use the language detected in recon to search for type definitions appropriately. Focus on domain-logic directories (not utilities, not framework types). Each type name is a term. Ask: would someone new to this codebase understand what this type represents?

### 2. Enum variants

Enums encode a fixed vocabulary. Each variant is a glossary candidate when the names are domain-specific:

```
# OrderStatus { Pending, Confirmed, Shipped, Delivered, Cancelled }
# UserRole { Member, Admin, Owner }
```

### 3. Existing glossaries

```
Grep: "glossary"
Grep: "terminology"
Grep: "definitions"
```

Check README, docs, and wikis for existing glossary sections.

### 4. Ambiguous terms in code

Search for terms that appear with different meanings in different places:

```
Grep: "user"   -- Is it always clear whether this means authenticated member, visitor, or admin?
Grep: "account" -- Does it mean user account, billing account, or organization?
Grep: "project" -- Is this consistent across the codebase?
```

### 5. Authorization and permission terminology

Authorization systems often introduce domain-specific vocabulary that is used inconsistently:

```
Grep: "role"
Grep: "permission"
Grep: "scope"
Grep: "claim"
Grep: "grant"
Grep: "privilege"
Grep: "entitlement"
```

Look for terms where the codebase's usage differs from common understanding:
- "role" vs. "permission" vs. "scope" -- are these used consistently?
- "admin" -- does it mean the same thing everywhere? (org admin vs. system admin vs. super admin)
- "owner" -- owner of what? (resource owner vs. account owner vs. org owner)
- "member" vs. "user" vs. "account" in authorization context

### 6. External sources

**Documentation**: Onboarding docs and domain model descriptions.

**Tickets**: Tickets where miscommunication was caused by ambiguous terminology.

## Debt signals

- **debt**: A term is used inconsistently across the codebase (e.g., "user" means "member" in one module and "any visitor" in another). The inconsistency is the debt; the correct definition is the truth.
- **ambiguous**: A term is used consistently but its definition is nowhere explicit.

## Writing candidates

Define the term precisely and state what it is NOT:

```
name: "Member"
content: "An authenticated individual who has completed onboarding
and has an active subscription. Has a profile, can create content.
Distinct from 'visitor' (unauthenticated). The term 'user' is
ambiguous in this codebase -- use 'member' or 'visitor' instead."
classification: truth
confidence: medium
reasoning: "Type system distinguishes Member from Visitor consistently.
However, some API endpoints use 'user' in their paths."
```
