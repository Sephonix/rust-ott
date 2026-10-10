# NIH-plug local compatibility fixes

Vendored `nih_plug` and its derive crate from
[robbert-vdh/nih-plug](https://github.com/robbert-vdh/nih-plug), revision
`de421011f41a6d10fc8c7a6084e4f4dee0143683` (ISC, see LICENSE).
Other NIH-plug crates remain pinned Git dependencies.

Only `src/wrapper/clap/wrapper.rs` has behavior changes:

- Reject empty or >16 MiB CLAP state length prefixes before allocation. The
  upstream loader trusts random/corrupt u64 lengths and aborts on allocation.
- Notify the host with `CLAP_PARAM_RESCAN_VALUES` after successful CLAP state
  restoration. The upstream state stream loader restores values without this
  notification, leaving hosts with stale parameter caches.

Both are covered by the full CLAP validator state tests (including random
state bytes and all three state reproducibility tests). The upstream Cargo
workspace list was reduced to the derive crate because only the core is vendored.
These fixes can be removed when equivalent changes land upstream.

Two diagnostic-only adjustments suppress upstream warnings on Rust 1.99:
`semicolon_in_expressions_from_non_local_macros` in `src/lib.rs`, and the
macro re-export's `unused_imports` in `src/wrapper/clap/util.rs`.
