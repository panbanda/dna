---
name: managing-labels
description: Registers and organizes label keys in DNA. Use when adding new metadata dimensions for filtering artifacts, or when setting up a new project with custom label keys.
---

# Managing Labels

Labels are key-value pairs attached to artifacts for filtering and organization. Unlike free-form tags, label keys must be registered before use. This ensures consistency across a project and prevents typos from fragmenting your label space.

## Listing registered labels

```bash
dna label list
```

## Viewing all kinds and labels

To see the full project context -- all registered kinds and labels together:

```bash
dna context
```

## Registering a new label

```bash
dna label add <key> "<description>"
```

The key must be:

- Lowercase letters, numbers, and hyphens only
- Not a reserved word

The description should explain what this label represents and give example values.

```bash
dna label add domain \
  "Domain or bounded context the artifact belongs to. Ex: auth, billing, orders, onboarding"

dna label add regulation \
  "Regulatory framework that requires this artifact. Ex: gdpr, hipaa, pci-dss, soc2"

dna label add priority \
  "Relative importance of the artifact. Values: high, medium, low"

dna label add provider \
  "External service provider name. Ex: stripe, sendgrid, google-maps"
```

## Showing label details

```bash
dna label show <key>
```

## Removing a label

```bash
dna label remove <key>
```

Removing a label key does not strip the label from existing artifacts. Those labels remain on the artifacts but are no longer part of the registered set.

## When to create a new label

Create a new label when:

- Multiple artifacts share a metadata dimension that is useful for filtering
- The values are enumerable or follow a known pattern
- You would use `--label key=value` in `dna list` or `dna search` queries

Do NOT create a new label when:

- The information belongs in `--context` (free-form semantic content)
- Only one or two artifacts would use it
- The dimension is better expressed as a separate kind

## Labels vs context

Labels are for exact-match filtering: `dna list --label domain=auth`.

Context is for semantic similarity: `--context "Part of the auth system, relates to GDPR"`.

Use both together for best results. Labels narrow the search space; context improves relevance ranking.
