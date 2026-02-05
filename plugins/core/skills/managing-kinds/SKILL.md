---
name: managing-kinds
description: Registers and organizes custom artifact kinds in DNA. Use when the built-in template kinds do not cover a domain concept, or when setting up a new project with custom truth categories.
---

# Managing Kinds

Kinds categorize truth artifacts. The intent template provides 11 kinds, the agentic template provides 5. You can register additional kinds for domain-specific needs.

## Listing registered kinds

```bash
dna kind list
```

## Registering a new kind

```bash
dna kind add <slug> "<description>"
```

The slug must be:

- 2-64 characters
- Lowercase letters, numbers, and hyphens only
- Not a reserved word (all, any, artifact, config, kind, kinds, none, search, system)

The description should follow the pattern of built-in kinds: state what this kind captures in one sentence, then give an example with `Ex:`.

```bash
dna kind add runbook \
  "Operational procedure for a specific failure mode. Ex: 'Database failover: promote read replica, update DNS, verify replication'"
```

## When to create a new kind

Create a new kind when:

- Multiple artifacts share a category that does not fit existing kinds
- The category represents a distinct type of truth (not just a label value)
- You would search for this category specifically with `--kind`

Do NOT create a new kind when:

- A label would suffice (`--label type=runbook` on an existing kind)
- The category is a subcategory of an existing kind (use labels instead)
- Only one or two artifacts would use it

## Kind naming

Use a singular noun that describes what the artifact IS:

- `runbook` (not `runbooks` or `operational-procedures`)
- `sla` (not `service-level-agreements`)
- `decision` (not `architectural-decisions`)

## MCP integration

When kinds are registered, the MCP server automatically generates tools for each kind:

- `dna_<kind>_search` -- semantic search within this kind
- `dna_<kind>_add` -- store a new artifact of this kind
- `dna_<kind>_list` -- list all artifacts of this kind

Hyphens in slugs become underscores in tool names.
