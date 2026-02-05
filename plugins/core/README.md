# Core Skills

Skills that teach AI agents how to use DNA as a truth layer during the development lifecycle.

## Skills

| Skill | Phase | What it does |
|-------|-------|-------------|
| `planning-with-truth` | Plan | Search DNA before choosing an approach. Surface relevant intents, constraints, and contracts that narrow the solution space. |
| `validating-approach` | Validate | After planning, check each step against known contracts, constraints, and evaluations before writing code. |
| `checking-truth` | Execute | Lightweight pre-change check. Search for artifacts related to the file or area being modified. |
| `capturing-artifacts` | Capture | How to write good artifacts. Quality tests, kind selection, label conventions. |
| `searching-artifacts` | Any | Query patterns and score interpretation for different search strategies. |
| `managing-kinds` | Setup | When and how to register custom artifact kinds. |

## Lifecycle

```
Plan -------> Validate -------> Execute -------> Capture
search DNA    check plan vs.    check before     record new
for context   known truth       each change      decisions
```

Every phase consults DNA. The planning phase is broadest (search everything relevant). Validation is targeted (does this plan violate known constraints?). Execution checks are narrow (does this file touch a governed area?). Capture records decisions that emerged during development.

## Usage

These skills are loaded automatically when the `dna-core` plugin is installed. They provide context to AI agents working in a DNA-initialized project.
