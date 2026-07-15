# wgpu_l3 glyphon patch

This directory is a source-pinned copy of `glyphon` 0.11.0 from crates.io
(registry checksum
`68dde9640ec6f986f59a265b4c0a5177f61e87e3ba71983b2195dab119cda0fa`). The
upstream license files are preserved beside this notice.

wgpu_l3 retains prepared text across semantic commits. Upstream
`TextAtlas::trim` clears the atlas live set, while upstream `TextRenderer`
0.11.0 exposes no capability for an already-prepared renderer to reassert the
glyph allocations referenced by its retained vertex buffer. Preparing unrelated
text after a trim can therefore evict allocations still used by active retained
text.

The local delta adds two retained capabilities:

- `TextRenderer::retain_prepared`, backed by a private
  `TextAtlas::retain_prepared` operation. Preparation records the opaque cache
  keys already used to build the renderer. Retention only checks those keys and
  marks their existing allocations live; it performs no shaping, rasterization,
  vertex reconstruction, or GPU upload.
- `Viewport::update_render_offset`, which uses the existing 16-byte viewport
  uniform to move prepared vertices in device space. One copy-on-write viewport
  property slot per live structural scroll value lets candidate preparation
  realize retained text without mutating the active slot, per-text preparation,
  or per-text uniform writes. Equal active/candidate values share the same slot.

Keep the delta limited to those capabilities. When updating glyphon, first
check whether upstream exposes equivalent retained-allocation ownership and a
bounded prepared-text transform. If it does, remove this source copy and return
`Cargo.toml` to a registry dependency after the renderer debug atlas-pressure
and retained-scroll witnesses pass against the upstream API.
