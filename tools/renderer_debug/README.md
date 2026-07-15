# Renderer debug oracle

`renderer_debug` is the renderer campaign's unpublished, development-only
oracle and benchmark crate. It compares two realizations of the same closed
scene fixture, reads their pixels back in-process, and reports the renderer work
that produced them. It is deliberately not a production renderer selector or a
second public rendering API.

Run every command below from the workspace root.

## Quick start

The pure comparator tests need no GPU:

```text
cargo test -p renderer_debug
```

List the closed fixture cases, compare every case at the four campaign scale
factors, or inspect one failure in detail:

```text
cargo run --release -p renderer_debug -- list
cargo run --release -p renderer_debug -- oracle-all
cargo run --release -p renderer_debug -- oracle ordered-group 1.25
```

The GPU acceptance witnesses are ignored during the ordinary workspace suite
because they require an available adapter. Run them explicitly in release
mode:

```text
cargo test --release -p renderer_debug -- --ignored --nocapture
```

Run one witness by its test-name substring while investigating a failure:

```text
cargo test --release -p renderer_debug control_gallery_property_tick_is_blend_equivalent_offscreen -- --ignored --nocapture
cargo test --release -p renderer_debug pending_semantic_realization_yields_to_exact_active_output -- --ignored --nocapture
```

Windows uses the framework's DX12-first adapter policy. An explicit
`WGPU_BACKEND` value remains authoritative. The optional software fallback rail
can be selected with `WGPU_L3_FORCE_FALLBACK_ADAPTER=1`; it is a correctness and
CPU-amplification witness, not an integrated-GPU performance proxy.

## CLI receipts

The binary exposes these code-owned observations:

```text
renderer_debug list
renderer_debug reference <case>
renderer_debug reference-all
renderer_debug oracle <case> <scale>
renderer_debug oracle-all
renderer_debug work <case>
renderer_debug retention <case>
renderer_debug partial-update
renderer_debug churn <iterations>
renderer_debug bench <case> <iterations>
```

Use `cargo run --release -p renderer_debug -- <arguments>` to invoke them.
`bench` performs one warmup and reports the requested measured sample count,
adapter, backend, operating system, architecture, p50, p95, and maximum. A
receipt is meaningful only when its case, scale, warmup, samples, environment,
and acceptance currency are preserved with the result.

`work`, `retention`, `partial-update`, and `churn` expose semantic work rather
than elapsed time alone: node realization, primitive and text preparation,
content/property uploads, resource lifetime, plan reuse, draw/pass topology,
surface sampling, opacity classification, and bounded effect storage. Prefer
literal-zero or structural assertions over comparative timing when the law is
zero work.

## Choosing a comparison policy

Start with `Tolerance::Exact`. Use `PerChannel` only when two proven-equivalent
floating blend routes can round differently. Use `Silhouette` only for a named
antialiased coverage law whose maximum channel delta and differing-pixel budget
are both justified.

Never increase a tolerance to hide a geometry, ordering, clipping, alpha,
packing, or resource-lifetime defect. The detailed single-case command and the
production-gallery witnesses report changed bounds, maximum-delta samples, and
nearby expected pixels. A matching pixel at a nearby coordinate points to
localization or snapping; it is not evidence for a looser blend tolerance.

When a transition fails, distinguish these questions before changing code:

1. Does the initial retained image already differ from the compatibility
   image?
2. Does only the property/activation transition differ?
3. Is the mismatch a uniform spatial displacement, a clip boundary, a blend
   delta, missing content, or incomplete output?
4. Did the semantic-work receipt remain within its zero-work and bounded-memory
   laws?

Fix the owner that first makes the two paths disagree. Do not special-case the
fixture in the adapter.

## Adding a witness

1. Construct one closed `scene::Commit` and one compatible complete
   `scene::Properties` snapshot. Carry composition identity; do not mint a
   renderer identity or hash flattened primitives.
2. Feed the exact fixture to the compatibility and retained realizations. The
   adapter may translate representation but may not change semantic input.
3. Cover 1.0, 1.25, 1.5, and 2.0 scale when physical snapping, text, clipping,
   or effects participate.
4. State the comparison policy and assert the relevant semantic-work,
   activation, resource, and cleanup currencies.
5. Keep GPU work as an ignored release witness and keep its pure comparison or
   contract logic in the ordinary suite where possible.
6. Preserve the narrow `renderer-debug` feature boundary. Production code must
   not depend on this crate, expose WGPU handles through it, or gain a runtime
   old/new selector.

The compatibility renderer and its adapter are temporary campaign scaffolding.
Checkpoint 9 deletes them with the old renderer. Direct readback, work,
topology, lifecycle, and benchmark witnesses for the sole retained renderer
survive.

## Evidence order and runtime smoke

Use evidence in this order:

1. deterministic contract/topology witnesses and this in-code oracle;
2. feature-gated owner-local counters or temporary logging;
3. an external profiler only when the first two cannot answer a named question
   and no practical code-owned witness exists;
4. optional reports from other hardware as corroboration.

No external machine, person, network service, or returned artifact is required
to run or close these witnesses.

Offscreen readback cannot detect process crashes, window pacing, incomplete
native presentation, or interaction feel. Any change to renderer topology,
presentation scheduling, or input projection also requires the campaign's real
release Control Gallery smoke: fast/large scrolling, continuous typing and
selection, representative dragging, process-survival observation, and explicit
closure of the gallery afterward.
