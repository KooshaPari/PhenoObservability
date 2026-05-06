# Specifications

## Acceptance Criteria

- The root README badge block links to `https://sladge.net`.
- The badge image source is `https://sladge.net/badge.svg`.
- The dirty detached canonical checkout remains untouched.
- projects-landing receives updated current-head proof.

## Assumptions, Risks, Uncertainties

- Assumption: This governance refresh is documentation-only.
- Risk: Broad Cargo validation may remain blocked by pre-existing workspace path dependency issues or local disk limits.
- Mitigation: Run focused docs validation and the cheapest Rust formatting gate, then record exact blockers.
