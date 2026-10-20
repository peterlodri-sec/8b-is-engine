# the language cousin — Spherepop, innermost-first, and history as identity

*Bridge lane. The engine's [admissibility](../crates/admissibility/src/lib.rs)
crate treats a world as a *geometry of admissible continuation*: what you may
do next is fixed by the attested history, not by how good the outcome looks.
**Spherepop** (Flyxion / `standardgalactic`) is that idea as a visual
language: nested parentheses drawn as nested bubbles, where only a bubble
with nothing left inside it may pop. This doc reflects what exists upstream,
kompresses what it gives the engine, and wires it into the constellation.*

---

## 1. reflect — what actually exists

- **The playground** — `standardgalactic.github.io/spherepop`. Type
  `(((1 + 2) + 3) + 4)`, see three nested circles, click the innermost one to
  pop it; evaluation runs inside-out. `()`, `[]` and `{}` mean the same thing.
- **`standardgalactic/spherepop`** — the canonical repo, mostly TeX plus
  specs. `SPEC.md` defines the normative semantics: the primitives **`Pop`,
  `Refuse`, `Bind`, `Collapse`** over an option space of object ids, with an
  **append-only event history**. `spherepop-kernel/` (Rust) is the reference
  implementation. A Python oracle and a C fixture kernel agree with it on the
  shared fixtures. Papers include *History as Identity*, *Event-Historical
  Aggregation* and *Computation as Semantic Maintenance*.
- **A second lineage** — the sphere-path / quotient grammar in
  `standardgalactic/personalinks`. The two have not been reconciled upstream
  (their issue #1), so this bridge does not pick a side. It takes only the
  part both share: pop inner before outer, and keep the history.
- **Licence: none declared.** The repo ships no `LICENSE` file. So **nothing
  is vendored**. `crates/spherepop` is a clean-room implementation written
  from the public description and playground behaviour. Where it differs from
  the canon, the canon (`SPEC.md`, `spherepop-kernel/`) wins.

## 2. kompress — what it gives the engine

[`crates/spherepop`](../crates/spherepop/src/lib.rs) has zero dependencies,
is deterministic, and is pinned by 11 unit tests plus a doctest:

1. **Innermost-first is admissibility.** `Sphere::admissible()` returns
   exactly the frontier of unpopped bubbles that have no unpopped children.
   That is the same shape as the Central Theorem's admissible-action set:
   what the current state permits, independent of the result. Popping
   outside that set does not panic and does not mutate anything. It returns
   a **`Refusal`** (`InnerFirst { blocking }`, `AlreadyPopped`,
   `DivisionByZero`, `Overflow`, `UnknownBubble`).
2. **A refusal is an event.** Admitted pops and refusals both go into the
   append-only `Event` log, mirroring the canon's `Pop` / `Refuse` pair. A
   failed attempt is part of what happened.
3. **History is identity.** `Sphere::replay(source, history)` rebuilds the
   exact state and rejects any history the source would not admit (for
   example, a forged pop value gives `Diverged { index }`).
   `Sphere::digest()` is FNV-1a over source + history. Pop order is
   *confluent* (any admissible schedule reaches the same value), but two
   schedules are still **different spheres**, and their digests differ.
4. **A render is a world.** `Sphere::render()` prints the current state as
   valid Spherepop source, with popped bubbles replaced by their values
   (`(((1 + 2) + 7) + [5 + 6])`). Every render re-parses to the same answer.
   That is the engine law in miniature: *one source → one state → every
   host is a render*.
5. **Safe by construction.** The playground evaluates with JavaScript's
   `Function()` constructor. This core is a recursive-descent parser over
   checked `i64` arithmetic, with standard precedence inside a bubble. No
   eval, no panics on user input.

`Bind` and `Collapse` from the canon are deliberately **not** modelled yet.
They need the option-space semantics of `SPEC.md`, and should be ported
against `spherepop-kernel/` fixtures once the licence question is settled.

## 3. wire — the constellation seam

- **Wired into** [admissibility](../crates/admissibility/src/lib.rs) via
  [`spherepop_instance.rs`](../crates/admissibility/src/spherepop_instance.rs).
  `structure_from_spheres(&spheres, observe)` maps each `Sphere`'s innermost
  frontier `Sphere::admissible()` into a `Structure<Obs, BubbleId>`, pinning
  CT-000 (frontier overlap), CT-000⁺ (coarse popped-count observation blurs
  which inner bubble remains), CT-015 (admissibility factors through
  `render()` across distinct pop-order and refusal histories), and CT-016
  (a refused pop owns an event-log position and shifts `digest()` while
  leaving `render()` intact).
- **Cousin to** [the cartography cousin](stipple-bridge.md) and
  [the CPU cousin](project-zero-bridge.md). All three are *one deterministic
  transform*: seed → tile, ternary weights → `i32` accumulator, and now
  source + history → sphere.
- **8b-is** — `8b-is/claimshift` holds the Lean view of the Central Theorem.
  A future `Spherepop.lean` could state the innermost-first frontier lemma
  next to it. Smart Tree's context tooling could treat a pop history as a
  replayable trace.
- **vaked** — next is a small playground surface on a vaked site: the same
  bubbles drawn from `render()`, with refusals shown as a soft shake rather
  than an error.
- **Upstream:** `github.com/standardgalactic/spherepop` (`SPEC.md`,
  `spherepop-kernel/`). Credit to Flyxion / standardgalactic for the
  language and the papers. Built by the 8b.is crew (Hue, Aye, Taisha, Omni)
  with family at the table (Alexandra, Chris).

*0 + 1 · pop the innermost · the history is the sphere · the weave is the loop.*
