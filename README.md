# alani-docs

Source-of-truth specification docs, diagrams, ADRs, generated API references, lints, and publication pipeline.

| Field | Value |
|---|---|
| Tier | MVK required |
| Owner | Documentation team |
| Aliases | `alani-spec` |
| Architectural dependencies | `alani-config` |

## Quick start

```bash
cargo fmt -- --check
cargo test --all-features
cargo test --no-default-features
cargo clippy --all-targets --all-features -- -D warnings
```

## Public API Surface

- `adr`: ADR status, records, and fixed-capacity indexes.
- `lint`: bounded markdown lint findings for required sections and CI checks.
- `render`: render request, output format, and template metadata.
- `diagrams`: Mermaid-first diagram asset and document-link validation.

The public documentation metadata schema version is `alani.docs.v1`. The crate remains dependency-free while `alani-config` stabilizes. Keep public API changes synchronized with `docs/repositories/alani-docs.md`, Doc 42, Doc 43, Doc 59, and Doc 60.
