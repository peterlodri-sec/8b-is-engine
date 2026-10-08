# the cartography cousin — Stipple, pattern fills, and the world renderer

*Bridge lane. The engine's renderer is a seeded, deterministic projection of
the world (`crates/world-core/src/render.rs`) — see
[renderer-candidates.md](renderer-candidates.md). **Stipple**
(`francoisbl/stipple`) is the same determinism in a different domain: a
runtime pattern engine that turns a *seed* into a seamless fill tile. This doc
reflects its state, kompresses what it gives the renderer, and wires it into
the constellation.*

---

## 1. reflect — what actually exists

- **`francoisbl/stipple`** — published as **`stipple-maplibre` v0.4.0**,
  **MIT**, TypeScript. Pattern-based polygon styling for **MapLibre GL JS**.
- **Five pattern families**, one engine: `solid`, `stipple`, `hachures`,
  `cross`, `grid`, `dots` (`src/engine/types.ts` → `PatternType`).
- **One deterministic call** — `makeTile(pattern, size, color, weight, angle)`
  returns a seamless RGBA `TileImage` (`.data: Uint8Array`). Sized by `size`
  (density control), weighted by `weight`, angled by `angle`.
- **No DOM dependency.** `makeTile` runs unchanged in the browser (native
  canvas) or in **Node** via `createMiniContext`, a small pure-JS rasterizer
  (`src/engine/miniContext.ts`). Same seed → same bytes, either host.
- Determinism comes from `mulberry32` + `hashStringToSeed`
  (`src/engine/seededRandom.ts`): a fixed seed reproduces a fixed arrangement.

## 2. kompress — what it gives the world renderer

The engine's law is *one seed → one world → every host is a render*. Stipple
supplies the **fill** half of that law for polygon/map surfaces:

1. **Density is distance, weight is emphasis.** A pattern tile is a scale —
   `size` up = lower density. That is a *legend* that lives in geometry, not a
   colour ramp. For terrain, land-use and category maps (the
   [renderer-candidates](renderer-candidates.md) shortlist already looks at
   Canvas map surfaces), it is a ready-made visual vocabulary.
2. **Seed-stable texture.** Because tiles are seed-derived and byte-stable,
   a woven world can be hashed: the same `.scene.py` export that Blender
   renders could carry a matching texture atlas without a shipped asset.
3. **No MapLibre required for the engine.** Only the *bindings*
   (`src/maplibre/*`) need MapLibre; `makeTile` is host-agnostic. That means
   the pattern engine is usable by the engine's **world-core SVG/Canvas
   renderer** and by any web surface — no map tiles, no API key.

## 3. wire — the constellation seam

- **`weave.vaked.dev`** — the live board: every constellation surface rendered
  as its own fill (stipple / hachure / dots / grid / cross), woven by
  `makeTile`. The vendored engine (`stipple.global.js`, IIFE global
  `MaplibrePatternFills`) is the same `stipple-maplibre` build.
- **Cousin to** the [ternary-model-lane](ternary-model-lane.md): both are
  *one deterministic transform, one scale applied once* — there it is
  accumulate-in-`i32`, here it is seed-in → tile-out.
- **Upstream:** `github.com/francoisbl/stipple` (MIT) — the isolated engine is
  the part worth borrowing; the MapLibre bindings are the part worth pointing
  a map surface at.

*0 + 1 · the map is the weave · the weave is the loop.*
