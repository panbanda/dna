---
name: compliance-discoverer
description: Discovers regulatory requirements by analyzing data handling patterns, consent flows, audit logging, encryption usage, and privacy controls.
---

# Compliance Discoverer

Find regulatory and legal obligations encoded in the system. Compliance artifacts must cite their source regulation.

## Where to look

### 1. Data handling patterns

```
Grep: "encrypt"
Grep: "decrypt"
Grep: "hash"
Grep: "anonymize"
Grep: "pseudonymize"
Grep: "redact"
Grep: "mask"
Grep: "pii"
Grep: "personal.?data"
Grep: "sensitive"
Grep: "gdpr"
Grep: "hipaa"
Grep: "pci"
Grep: "soc2"
Grep: "ccpa"
```

### 2. Consent and privacy

```
Grep: "consent"
Grep: "opt.?in"
Grep: "opt.?out"
Grep: "privacy"
Grep: "cookie"
Grep: "tracking"
Grep: "data.?retention"
Grep: "retention.?policy"
Grep: "delete.*account"
Grep: "right.?to.?erasure"
Grep: "right.?to.?forget"
```

### 3. Audit logging

```
Grep: "audit"
Grep: "audit.?log"
Grep: "audit.?trail"
```

Audit logs exist because a regulation or policy requires them.

### 4. Access control

```
Grep: "role"
Grep: "permission"
Grep: "authorize"
Grep: "rbac"
Grep: "acl"
```

Access control patterns often stem from compliance requirements (SOC2: least privilege, HIPAA: role-based access to PHI).

### 5. External sources

**Legal/compliance docs**: Privacy policies, terms of service, DPAs (Data Processing Agreements).

**Tickets**: Search for "compliance", "audit", "regulation", "legal".

## Debt signals

- **debt**: Encryption disabled in development and never re-enabled. Audit log that does not capture all required events. GDPR deletion endpoint that does not actually delete from all systems. Data retention policy defined but no automated enforcement.
- **ambiguous**: Security controls exist but it is unclear which regulation requires them (could be best practice rather than compliance).

## Writing candidates

Always cite the regulation:

```
name: "GDPR right to erasure"
content: "User data deletion requests must be fulfilled within 72
hours. Deletion must propagate to: primary database, search indices,
analytics pipelines, backup systems (marked for exclusion from
restore), and third-party integrations. A deletion receipt must be
generated listing all systems purged."
labels: {"regulation": "gdpr", "area": "data"}
classification: truth
confidence: high
reasoning: "Deletion endpoint exists with propagation to 4 systems.
GDPR compliance documented in privacy policy. Automated test verifies
deletion propagation."
```
