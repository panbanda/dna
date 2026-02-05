---
name: writing-algorithms
description: Creates algorithm artifacts that describe computation logic without binding to a specific implementation. Use when codifying formulas, pricing rules, scoring logic, or any computation where the logic itself is the truth, independent of the code that implements it.
---

# Writing Algorithms

An algorithm artifact captures HOW a computation works -- the logic, not the code. It is the kind that sits closest to implementation, so extra care is needed to keep it at the right level of abstraction.

## What an algorithm IS

A computation rule: one formula, threshold, or decision procedure. It answers: "What is the correct result for a given input?"

## What an algorithm is NOT

- Not code (no function signatures, no language-specific syntax)
- Not an intent (algorithms describe HOW, intents describe WHY)
- Not a test case (evaluations define success criteria; algorithms define the logic)

## Structure

1. **The formula or rule** in plain language or mathematical notation
2. **Input definitions** -- what goes in, with types and ranges
3. **Output definition** -- what comes out
4. **Edge cases** with explicit expected behavior
5. **The reason** for non-obvious choices

## Examples

**Good:**

```bash
dna add algorithm \
  "Line item total = unit price * quantity * (1 - discount rate).
Order total = sum of line item totals.
Tax is calculated on the order total AFTER discounts.
Shipping is calculated on the PRE-discount subtotal to prevent gaming via high-discount items.
Final price = order total + tax + shipping.

Edge cases:
- 100% discount: line item total is 0, but shipping still applies
- Negative quantity: rejected at input validation, never reaches pricing
- Currency: all arithmetic in integer cents, convert to display format only at render
- Rounding: round each line item to nearest cent before summing" \
  --name "Order price calculation" \
  --label area=billing \
  --format markdown
```

An agent implementing this in any language produces the same results. The edge cases section captures decisions that would otherwise require institutional knowledge.

```bash
dna add algorithm \
  "Risk score = (0.4 * transaction_amount_factor) + (0.3 * velocity_factor) + (0.2 * geo_factor) + (0.1 * device_factor).

Each factor is normalized to [0.0, 1.0]:
- transaction_amount_factor: amount / account_average_90d, capped at 1.0
- velocity_factor: transactions_last_hour / 10, capped at 1.0
- geo_factor: 1.0 if country differs from last 3 transactions, else 0.0
- device_factor: 0.0 if known device, 0.5 if new device, 1.0 if impossible device fingerprint

Thresholds:
- score < 0.3: approve automatically
- 0.3 <= score < 0.7: flag for review
- score >= 0.7: block and notify" \
  --name "Transaction risk scoring" \
  --label area=fraud
```

**Bad:**

```rust
fn calculate_price(items: &[Item]) -> f64 {
    items.iter().map(|i| i.price * i.qty as f64 * (1.0 - i.discount)).sum()
}
```

This is code, not an algorithm artifact. It dies with a language change, has no edge case documentation, and does not explain the discount-then-sum ordering.

```text
Use a weighted average for risk scoring.
```

Too vague. Two agents would produce different implementations. The weights, normalization, and thresholds must be specified.

## The reason behind non-obvious choices

When an algorithm makes a choice that someone might want to "fix":

```text
Shipping uses PRE-discount subtotal.
Reason: prevents gaming via high-discount items where the customer
adds a heavily discounted item to reduce shipping cost below what
it would be for their actual purchase.
```

Without the reason, a future agent sees "pre-discount" and thinks "this is probably a bug -- should be post-discount."

## Precision matters

Specify rounding behavior, integer vs. floating point, and unit conversions explicitly. These are where most computation bugs live:

- "All money arithmetic in integer cents" prevents floating point errors
- "Round each line item before summing" vs. "sum then round" produces different totals
- "Timestamps in UTC" prevents timezone bugs

## During planning

When your plan involves any computation:

```bash
dna search "<the computation>" --kind algorithm
```

If an algorithm artifact exists, implement it exactly. Do not improve or optimize the formula without updating the artifact first.
