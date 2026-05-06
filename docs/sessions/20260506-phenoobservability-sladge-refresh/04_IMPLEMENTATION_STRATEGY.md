# Implementation Strategy

## Approach

Add one badge entry to the existing README badge block and leave all Rust, Python, and configuration files untouched.

## Boundary Decisions

- Do not modify the canonical detached checkout or its unrelated `deny.toml` change.
- Do not prune stale worktree metadata during this badge-only lane.
- Do not resolve duplicate pinned-reference blocks in README as part of this scoped refresh.
