---
name: sps-add-plugin
description: Add a DSP plugin to the Sephonix Plugin Suite workspace, including crate boundaries, host identities, build registration, packaging, and validation. Use when adding a plugin or integrating a new plugin implementation.
---

# Add a suite plugin

Read the repository's `AGENTS.md` and `docs/architecture.md`. Run commands from
the workspace root. `plugins/sps-ott` is the working reference, with an exported
`sps-ott` CLAP/VST3 package and separate DSP, GUI, and LV2 packages.

Place the new implementation under `plugins/sps-<name>/`. Give its exported
package that name and use the plugin prefix for helper packages. Keep engines,
parameter tables, factory values and meter layouts local. Use `crates/sps-dsp`
for gain mapping, crossovers, `params::Spec` and `AtomicValue`; do not import OTT
tuning into another plugin or make the shared crate depend on an engine.
Build the DSP independently of format APIs and UI. Share one plugin engine and
editor among its format adapters.

When adapting OTT as a starting point, replace its DSP and editor to match the
requested effect and choose distinct CLAP ID, 16-byte VST3 class ID, LV2 plugin
and UI URIs, and preset URI. Update all matching metadata and smoke-test IDs.
Existing OTT host identities must remain unchanged. Define the new plugin's
stable parameter IDs and port indices before publishing. Keep raw host/window
pointers and lifecycle handling in the adapters.

Integrate the implementation at these registration points:

- Root `Cargo.toml` members/dependencies and inherited package metadata.
- Root `bundler.toml` exported package/display name.
- `xtask/src/main.rs::PLUGINS`, with a separate plugin schema generator module
  and generated files under `plugins/sps-<name>/metadata`. The current registry
  expects CLAP/VST3 and Linux LV2; adapt it for a different requested format set.
- Exported package `[package.metadata.sps].bundles`, used by
  `python3 scripts/package.py sps-<name>`.
- Plugin manual, suite README catalog, relevant architecture documentation,
  and `.github/workflows/linux.yml` validation/archive upload entries.

Cover the effect's observable audio behavior, invalid controls/input, supported
channel/sample-rate configurations, reset, automation, and process allocations.
Create a runtime smoke test for the new exports rather than running OTT's test
against a different plugin. Build and validate the formats that are actually
supported; do not claim platform support from shared DSP portability alone.
