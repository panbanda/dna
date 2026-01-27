# Pull Request

## Description

<!-- Provide a brief description of the changes in this PR -->

## Related Issue

Closes #(issue number)

## Type of Change

- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Code refactoring

## Intent Artifacts

<!-- List any intent artifacts created for this change -->

- [ ] Intent artifact created (if new feature)
- [ ] Invariants documented (if system constraints)
- [ ] Contracts specified (if API changes)

## Testing

<!-- Describe the tests you've added or modified -->

- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Edge cases covered
- [ ] Code coverage ≥ 65%

### Test Coverage

```
# Paste output of: cargo tarpaulin --out Stdout
```

## Checklist

### Code Quality

- [ ] Code follows style guidelines (rustfmt)
- [ ] No clippy warnings (`cargo clippy --all-features --all-targets -- -D warnings`)
- [ ] All tests pass (`cargo test`)
- [ ] Documentation updated (if needed)
- [ ] CHANGELOG.md updated
- [ ] No new dependencies (or justified in description)

### Security

- [ ] No secrets or credentials in code
- [ ] Input validation implemented
- [ ] Security audit passed (`cargo audit`)
- [ ] Dependency licenses checked (`cargo deny check`)

### Documentation

- [ ] Public APIs have doc comments
- [ ] Examples provided (if applicable)
- [ ] README.md updated (if needed)
- [ ] Architecture docs updated (if needed)

## Performance Impact

<!-- Describe any performance implications -->

- [ ] No performance impact
- [ ] Performance improved
- [ ] Performance regression (justified below)

## Breaking Changes

<!-- If breaking changes, describe migration path -->

None / Described below:

## Screenshots/Examples

<!-- If applicable, add screenshots or usage examples -->

## Additional Context

<!-- Add any other context about the PR here -->

## Review Notes

<!-- Anything specific you want reviewers to focus on? -->
