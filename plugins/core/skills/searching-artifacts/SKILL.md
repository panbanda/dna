---
name: searching-artifacts
description: Performs semantic and filtered searches across DNA truth artifacts. Use when looking for relevant decisions, constraints, or contracts before implementing features, resolving ambiguity, or understanding why something works the way it does.
---

# Searching Artifacts

DNA uses vector embeddings for semantic search. Queries are natural language, not keywords.

## Search patterns

### Broad discovery

When exploring an unfamiliar area:

```bash
dna search "how does authentication work in this system"
dna search "what are the rules for billing"
```

Natural language queries work well because the embedding model understands meaning, not just word overlap.

### Kind-filtered search

When you know what type of truth you need:

```bash
dna search "payment processing" --kind contract
dna search "password requirements" --kind constraint
dna search "order total calculation" --kind algorithm
```

Kind filters narrow the search space. Use them when you want a specific category of truth.

### Listing with filters

When you want to browse rather than search:

```bash
dna list --kind intent
dna list --kind constraint --label domain=auth
```

Listing does not use semantic similarity -- it returns all matching artifacts.

### Retrieving a specific artifact

When you know the ID (from a previous search or reference):

```bash
dna get <artifact-id>
```

## Interpreting similarity scores

Search results include a similarity score (0.0 to 1.0, cosine similarity).

| Score | Interpretation |
|-------|---------------|
| > 0.8 | Strong match. Almost certainly relevant. |
| 0.6 - 0.8 | Good match. Read and assess applicability. |
| 0.4 - 0.6 | Weak match. May be tangentially related. |
| < 0.4 | Noise. Probably not relevant. |

These thresholds depend on the embedding model. The local model (bge-small) tends to produce lower scores than OpenAI models for equivalent relevance.

## Search strategies

### Before planning

Cast a wide net:

```bash
dna search "<the task or feature>" --limit 10
```

Read everything with score > 0.5. You are building a picture of what truth exists in this area.

### Before a specific change

Targeted search:

```bash
dna search "<the specific thing you are changing>" --kind contract --limit 5
dna search "<the specific thing you are changing>" --kind constraint --limit 5
```

Focus on contracts and constraints -- these are the artifacts most likely to conflict with a change.

### When resolving ambiguity

Search for glossary terms when the meaning of a concept is unclear:

```bash
dna search "<the ambiguous term>" --kind glossary
```

If no glossary artifact exists, the term is undefined. Consider capturing a definition.

### When no results are found

No results means either:

1. No truth has been captured for this area (common in early projects)
2. Your query does not match the stored content semantically (try rephrasing)

Try alternative phrasings before concluding no truth exists:

```bash
dna search "user authentication"
dna search "login flow"
dna search "session management"
```

If multiple phrasings return nothing, the area is genuinely uncovered.
