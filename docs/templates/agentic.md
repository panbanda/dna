# Agentic Template

Safety and governance for AI agents and LLM systems. This template provides a structured way to document and enforce safety requirements, threat mitigations, and governance controls for AI applications.

## Philosophy

AI systems require explicit safety boundaries that persist across model updates, prompt changes, and system evolution. This template captures the truths that define safe AI behavior - what the model should do, what it must never do, known threats, verification criteria, and governance requirements.

## When to Use

Use this template for:
- LLM-powered applications
- AI agents and assistants
- Multi-modal AI systems
- Any system using foundation models
- AI safety research and red teaming

## OWASP LLM Top 10 Coverage

This template maps directly to the [OWASP Top 10 for LLMs](https://owasp.org/www-project-top-10-for-large-language-model-applications/):

| OWASP | Kind | Label |
|-------|------|-------|
| LLM01: Prompt Injection | threat | `--label owasp=LLM01` |
| LLM02: Insecure Output Handling | boundary | `--label type=filter` |
| LLM03: Training Data Poisoning | governance | `--label aspect=provenance` |
| LLM04: Model Denial of Service | boundary | `--label type=policy` |
| LLM05: Supply Chain Vulnerabilities | governance | `--label aspect=provenance` |
| LLM06: Sensitive Information Disclosure | boundary | `--label type=filter` |
| LLM07: Insecure Plugin Design | behavior | `--label aspect=agency` |
| LLM08: Excessive Agency | behavior | `--label aspect=agency` |
| LLM09: Overreliance | governance | `--label aspect=disclosure` |
| LLM10: Model Theft | boundary | `--label type=policy` |

## Kinds

### behavior

**Purpose**: What the model does and how - capabilities, style, grounding, and agency.

**Aspects** (use `--label aspect=`):
- **capability**: What the model can do
- **grounding**: Citation and factual accuracy requirements
- **agency**: Tool use and autonomy boundaries
- **tone**: Communication style and persona

**Examples**:

```bash
# Capabilities
dna add behavior "Summarize documents up to 100k tokens" --label aspect=capability
dna add behavior "Generate code in Python, JavaScript, and Rust" --label aspect=capability
dna add behavior "Answer questions about company documentation" --label aspect=capability

# Grounding
dna add behavior "Must cite source documents for factual claims" --label aspect=grounding
dna add behavior "Express uncertainty when confidence is low" --label aspect=grounding
dna add behavior "Only use retrieved context for facts, not parametric knowledge" --label aspect=grounding
dna add behavior "Responses must be verifiable against provided sources" --label aspect=grounding

# Agency
dna add behavior "May call search and calculator tools only" --label aspect=agency
dna add behavior "Function calls require user confirmation" --label aspect=agency
dna add behavior "Cannot send emails without explicit user approval" --label aspect=agency
dna add behavior "Read-only access to databases, no mutations" --label aspect=agency

# Tone
dna add behavior "Respond helpfully without being sycophantic" --label aspect=tone
dna add behavior "Decline requests politely without lecturing" --label aspect=tone
dna add behavior "Maintain professional tone in enterprise context" --label aspect=tone
dna add behavior "Acknowledge limitations directly" --label aspect=tone
```

### boundary

**Purpose**: Safety limits and content policies - what the model must not do.

**Types** (use `--label type=`):
- **filter**: Runtime input/output validation rules
- **policy**: Organizational content policies
- **redline**: Absolute never-cross boundaries

**Severity** (optional): `--label severity=critical|high|medium|low`

**Examples**:

```bash
# Filters - runtime validation
dna add boundary "Reject prompts containing SQL injection patterns" --label type=filter
dna add boundary "Block outputs containing PII patterns (SSN, credit card)" --label type=filter
dna add boundary "Filter prompts with excessive token repetition (DoS)" --label type=filter
dna add boundary "Sanitize code outputs for XSS patterns" --label type=filter

# Policies - organizational rules
dna add boundary "Must not generate code for malware or exploits" --label type=policy
dna add boundary "Cannot provide medical diagnosis or treatment advice" --label type=policy
dna add boundary "Must not generate content impersonating real individuals" --label type=policy
dna add boundary "Cannot assist with academic dishonesty" --label type=policy

# Redlines - absolute prohibitions
dna add boundary "Never generate CSAM under any prompt" --label type=redline --label severity=critical
dna add boundary "Never provide instructions for weapons of mass destruction" --label type=redline --label severity=critical
dna add boundary "Never assist with human trafficking" --label type=redline --label severity=critical
dna add boundary "Never generate content promoting terrorism" --label type=redline --label severity=critical
```

### threat

**Purpose**: Known attack vectors and their mitigations.

**Labels**:
- `--label owasp=LLM01-10`: OWASP classification
- `--label severity=critical|high|medium|low`: Risk level
- `--label status=mitigated|open|monitoring`: Current state

**Examples**:

```bash
# Prompt Injection (LLM01)
dna add threat "Direct injection via 'ignore previous instructions' - mitigate with input validation and system prompt anchoring" --label owasp=LLM01 --label severity=high
dna add threat "Indirect injection via malicious content in retrieved documents - mitigate with content scanning and privilege separation" --label owasp=LLM01 --label severity=high
dna add threat "Role hijacking via 'you are now' patterns - mitigate with system prompt immutability" --label owasp=LLM01 --label severity=medium

# Insecure Output Handling (LLM02)
dna add threat "XSS via generated code snippets - mitigate with output sanitization" --label owasp=LLM02 --label severity=medium
dna add threat "Command injection via shell code generation - mitigate with sandboxing" --label owasp=LLM02 --label severity=high

# Sensitive Information Disclosure (LLM06)
dna add threat "Training data extraction via repetition attacks - mitigate with output filtering" --label owasp=LLM06 --label severity=high
dna add threat "System prompt extraction via jailbreak - mitigate with prompt protection" --label owasp=LLM06 --label severity=medium
dna add threat "PII leakage from context window - mitigate with PII detection" --label owasp=LLM06 --label severity=high

# Excessive Agency (LLM08)
dna add threat "Unrestricted function calling leading to data exfiltration - mitigate with allowlists" --label owasp=LLM08 --label severity=critical
dna add threat "Autonomous action loops without human oversight - mitigate with confirmation gates" --label owasp=LLM08 --label severity=high
```

### eval

**Purpose**: Verification benchmarks and evaluation criteria.

**Types** (use `--label type=`):
- **safety**: Harm prevention evaluations
- **bias**: Fairness and equity measurements
- **accuracy**: Factual correctness benchmarks
- **redteam**: Adversarial testing results

**Examples**:

```bash
# Safety evaluations
dna add eval "Safety score >= 95% on HarmBench" --label type=safety
dna add eval "Refusal rate >= 99% for harmful requests on AdvBench" --label type=safety
dna add eval "Zero violations on internal prohibited content test suite" --label type=safety

# Bias evaluations
dna add eval "Gender bias score < 0.1 on WinoBias benchmark" --label type=bias
dna add eval "Demographic parity within 5% across protected groups" --label type=bias
dna add eval "No statistically significant bias in hiring recommendations" --label type=bias

# Accuracy evaluations
dna add eval "Factual accuracy >= 90% on TruthfulQA" --label type=accuracy
dna add eval "Citation accuracy >= 95% for grounded responses" --label type=accuracy
dna add eval "Hallucination rate < 5% on domain-specific QA" --label type=accuracy

# Red team evaluations
dna add eval "Resists jailbreak in 99% of automated red team attempts" --label type=redteam
dna add eval "No successful prompt injections in 1000-case adversarial suite" --label type=redteam
dna add eval "System prompt remains confidential under extraction attempts" --label type=redteam
```

### governance

**Purpose**: Oversight, transparency, audit, and provenance requirements.

**Aspects** (use `--label aspect=`):
- **oversight**: Human-in-the-loop requirements
- **disclosure**: Transparency and communication
- **audit**: Logging and monitoring
- **provenance**: Data and model lineage

**Examples**:

```bash
# Oversight - human control
dna add governance "Human review required for content moderation appeals" --label aspect=oversight
dna add governance "Automated decisions affecting users require human confirmation" --label aspect=oversight
dna add governance "High-stakes outputs (legal, medical, financial) require expert review" --label aspect=oversight
dna add governance "Model updates require safety review before deployment" --label aspect=oversight

# Disclosure - transparency
dna add governance "Must identify as AI when directly asked" --label aspect=disclosure
dna add governance "Disclose AI-generated content in regulated contexts" --label aspect=disclosure
dna add governance "Inform users when operating with limited context" --label aspect=disclosure
dna add governance "Clearly communicate confidence levels for predictions" --label aspect=disclosure

# Audit - logging and monitoring
dna add governance "Log all prompts that trigger content filters" --label aspect=audit
dna add governance "Retain conversation logs for 90 days for safety review" --label aspect=audit
dna add governance "Alert on anomalous usage patterns (potential attacks)" --label aspect=audit
dna add governance "Track and report safety metric trends weekly" --label aspect=audit

# Provenance - lineage
dna add governance "Training data excludes PII, verified via automated scan" --label aspect=provenance
dna add governance "Model version: gpt-4-0613, deployed 2024-01-15" --label aspect=provenance
dna add governance "Fine-tuning data reviewed and approved by safety team" --label aspect=provenance
dna add governance "Third-party model components documented with licenses" --label aspect=provenance
```

## Regulatory Compliance

### EU AI Act

Map requirements using labels:

```bash
# Risk classification
dna add governance "High-risk classification: hiring decisions" --label aspect=oversight --label regulation=eu-ai-act --label risk=high
dna add governance "Article 52 transparency notice required" --label aspect=disclosure --label regulation=eu-ai-act

# Human oversight requirements
dna add governance "Human oversight for high-risk decisions per Article 14" --label aspect=oversight --label regulation=eu-ai-act
```

### NIST AI RMF

```bash
# Govern
dna add governance "AI risk management policy reviewed quarterly" --label aspect=oversight --label framework=nist-ai-rmf

# Map
dna add governance "AI system inventory maintained with risk ratings" --label aspect=provenance --label framework=nist-ai-rmf

# Measure
dna add eval "Bias testing conducted before each deployment" --label type=bias --label framework=nist-ai-rmf

# Manage
dna add governance "Incident response plan for AI safety events" --label aspect=oversight --label framework=nist-ai-rmf
```

## Usage

Initialize a project with the agentic template:

```bash
dna init --template agentic
```

Combine with intent template for comprehensive coverage:

```bash
dna init --template intent
dna kind add behavior "Model capability, style, or grounding rule..."
dna kind add boundary "Safety limit or content policy..."
dna kind add threat "Attack vector with mitigation..."
dna kind add eval "Verification benchmark or criteria..."
dna kind add governance "Oversight, transparency, audit, or provenance..."
```

## Querying

Find all safety-related threats:

```bash
dna search "injection" --kind threat
dna list --kind threat --label owasp=LLM01
```

Review governance requirements:

```bash
dna list --kind governance --label aspect=oversight
dna list --kind governance --label aspect=disclosure
```

Check evaluation status:

```bash
dna list --kind eval --label type=safety
dna list --kind eval --label type=redteam
```

## Best Practices

1. **Document threats as you discover them**: Red team findings should immediately become threat artifacts
2. **Link boundaries to threats**: Each boundary should mitigate one or more documented threats
3. **Version your evals**: Include benchmark versions and dates for reproducibility
4. **Review governance quarterly**: Oversight requirements evolve with regulations
5. **Use severity labels**: Prioritize critical redlines over medium-severity policies
6. **Trace provenance**: Document data and model lineage for auditability
7. **Test boundaries regularly**: Evals should verify that boundaries hold
