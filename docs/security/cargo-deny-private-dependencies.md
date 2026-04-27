# cargo-deny private dependency routing

## Context

`PhenoObservability` has two sibling Rust dependencies:

- `../pheno/crates/phenotype-errors`
- `../../../phenotype-bus`

`KooshaPari/pheno` is public, but `KooshaPari/phenotype-bus` is private. The
default GitHub Actions `GITHUB_TOKEN` for `PhenoObservability` cannot read that
sibling private repository, so `cargo metadata` and `cargo-deny` cannot resolve
the workspace in a clean CI checkout without an explicit read credential.

## Required Secret

Create repository secret `PHENOTYPE_REPO_READ_TOKEN` in
`KooshaPari/PhenoObservability`.

The token should be one of:

- a fine-grained PAT with read-only `contents` access to `KooshaPari/phenotype-bus`
- a GitHub App installation token with read-only access to `KooshaPari/phenotype-bus`

Do not use a broad personal token when a repository-scoped or app-scoped token is
available.

## Why the dependency was not rerouted

Changing `phenotype-bus` to a `git = ...` dependency does not remove the private
access requirement; Cargo would still need credentials to fetch the private repo.

Routing to `PhenoProc` is also not equivalent yet. `PhenoProc` currently has a
different `phenotype-event-bus` crate surface, while `phenotype-bus` remains the
typed async pub/sub crate consumed by the observability crates.

## Validation

After the secret is configured, the `cargo-deny` workflow checks out:

- `PhenoObservability` at `./PhenoObservability`
- `pheno` at `./pheno`
- `phenotype-bus` at `./phenotype-bus`

This layout matches the existing path dependencies and allows:

```bash
cargo deny check --manifest-path PhenoObservability/Cargo.toml
```
