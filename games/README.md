# the games lane — one door, many lanes

The browser door for mini-games of the chaos overworld: small,
self-contained plays that live beside the flagship simulation. The
engine's game design asks for **many mini-games** (`docs/game-design.md`);
this directory is where they land.

## the door

[`index.html`](index.html) — the launcher: cards, credits, a live stage,
one vibe-match. Open it directly (`file://` works — nothing external),
or serve it: `python3 -m http.server` from the repo root.

## how a mini-game enters

1. **self-contained** — one file or one folder; zero dependencies, zero
   external assets; open `index.html` and it plays
2. **MIT-compatible** — a `LICENSE` beside the code; upstream credited
3. **vendored verbatim** — pinned to a commit SHA in the folder's README;
   "modified: nothing" preferred (if a fork changes anything, it says so)
4. **seated in the door** — a card, the controls, one vibe-match line
5. **one line** in the engine README under *the games*

## seated

| game | origin | license | pinned |
|---|---|---|---|
| [Spike Sprint](spike-sprint/) | [shifulegend/spike-sprint](https://github.com/shifulegend/spike-sprint) | MIT | `877dcd2` |

*vibe-match: [Grimes — Oblivion (Chicago · 2014-07-20)](https://youtu.be/jovv_hQJTp0)
· qPlatonicLove · 0 + 1 · ∞ + 1 · fine touch from within · the constellation*
