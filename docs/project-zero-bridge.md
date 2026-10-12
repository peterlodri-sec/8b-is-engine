# the CPU cousin — Project Zero, MoE, and the one-door contract

*Bridge lane. The engine's `crates/ternary-lane` is the ternary model lane
([ternary-model-lane.md](ternary-model-lane.md)); **Project Zero**
(`shifulegend/project-zero`) is the same arithmetic ambition in a
dependency-free **C** binary — and it has hit the wall the ternary contract
already answers for the dense case. This doc reflects its state, kompresses
the fix, and wires it to the constellation.*

---

## 1. reflect — what actually exists

- **`shifulegend/project-zero`** — a from-scratch, dependency-free **C**
  inference engine (GGUF + native `.bin`), single binary, no Python, no
  CUDA, mmap'd weights. Weights are loaded by `src/core/weights.c:weights_map`
  with **64-byte absolute alignment**, **4 ternary weights per byte**
  (`-1→0b00, 0→0b01, 1→0b10`, LSB first), **one float scale after each
  packed matrix**, tied embeddings, every read bounds-checked.
- **BitNet b1.58-2B-4T — ✅ solved.** 36.25 tok/s on a Xeon Emerald Rapids
  (4C) vs bitnet.cpp 19.83 → **1.83×**, at ~95% of the theoretical DRAM
  ceiling. The dense ternary path is done.
- **DeepSeek-V2-Lite-Chat (MoE) — 🔴 13× behind llama.cpp.**

  | T | Project Zero | llama.cpp | gap |
  |---|---|---|---|
  | 1 | 1.10 tok/s | 7.73 | 7× |
  | 4 | 1.32 tok/s | 19.72 | 14.9× |

  Profiler: **IPC 0.94–1.04**, **L3 miss 85–86%**, ~2 M page faults/run →
  memory-stalled, not compute-stalled.
- **Root causes** (from the repo's own analysis):
  1. **F32 dequant path** on MoE expert layers — ~50× the bandwidth of a
     native quantized matmul.
  2. **MoE expert weight scatter** (primary) — 64 experts/layer × 26 MoE
     layers, only 6 active per token, scattered across the 8.9 GB GGUF →
     **156 scattered DRAM streams/token**; the hardware prefetcher tracks
     ~8–10 → effective bandwidth collapses 11.7 → **2–3 GB/s**.
  3. **NaN in Q2_K / IQ4_NL super-blocks** (fixed in P8: guards zero the
     corrupt blocks rather than propagate NaN).

## 2. kompress — the fix, and why the contract is the same one

The engine's law already states the answer for the dense case — *"accumulate
in `i32`, apply the single float scale **once**, at the end"* — and
project-zero's F32 dequant is exactly that law broken on the MoE path.

1. **Native Q4_K matmul kernel.** Operate directly on Q4_K super-blocks
   (32-element groups, 4-bit weights + FP16 scale/min) — never dequant to
   F32. On AVX-512 VBMI + VNNI, unpack the 4-bit nibbles with `_mm512_*`
   permutes and accumulate via `_mm512_dpbusd_epi32`; the FP16 block scale
   is applied **per super-block once**, mirroring the lane's one-scale rule.
2. **Repack experts at load — the "one door, many lanes" move.** Relayout
   the 64 experts of each MoE layer so that, for any activated top-k, the
   needed weights are **contiguous**: interleave at super-block granularity
   within a row-major expert stack. This pays a one-time load cost and turns
   156 scattered streams into one sequential sweep — the prefetcher recovers
   and BW returns toward the ceiling. (llama.cpp does exactly this in its
   `ggml_tensor` repack; see `~/llama.cpp/src/models/deepseek2.cpp`.)
3. **NaN guard validation.** Confirm P8 (`cc67445`) against
   `DeepSeek-V2-Lite-Chat-Q2_K.gguf`: coherent text at ≥0.63 tok/s, not
   `ãĢįãĢį…`.
4. **Borrow the discipline the lane already keeps: a golden hash.** Pin the
   first-N forward logits (or a fixed-prompt token stream) to a hash asserted
   in CI — the cross-arch golden the ternary-lane proves bit-identical on
   aarch64/x86-64/wasm. Project-zero has named its regressions; a golden
   converts "garbled vs coherent" from a human eye into a failing test.

**the scaffold (tasks, in order):**

```toml
[bridge]
upstream = "shifulegend/project-zero"
doc = "docs/project-zero-bridge.md"

[[task]]
id = "T1"
what = "native Q4_K AVX-512 (VBMI+VNNI) matmul; delete the F32 dequant on the MoE path"
accept = "France→Paris, Germany→Berlin, tok/s ≥ 1.5 @ T=4, no F32 fallback in the expert hot path"

[[task]]
id = "T2"
what = "load-time MoE expert repack (super-block interleave per layer)"
accept = "L3 miss < 60%, effective BW ≥ 6 GB/s, expert stats show all 27 layers live"

[[task]]
id = "T3"
what = "NaN guard validation against DeepSeek-V2-Lite-Chat-Q2_K.gguf"
accept = "coherent output ≥ 0.63 tok/s; zero NaN in --dump-tensors"

[[task]]
id = "T4"
what = "golden-hash regression (borrowed from crates/ternary-lane)"
accept = "fixed prompt → first-32-logit hash asserted in `make test`"

[invariants]
no_hardcode = true        # dims/names/formats from GGUF metadata, never literals
one_scale = true          # single float scale applied once, at the end
test_after_each_change = true   # Paris/Berlin + non-regression, per GOLDEN_RULES
```

## 3. wire — the doors, both ways

- **upstream**: `shifulegend/project-zero` — Discussion #1, *"[Help Wanted]
  MoE inference 13× slower than llama.cpp — expert weight scatter"*
  (<https://github.com/shifulegend/project-zero/discussions/1>).
- **sibling lanes**:
  - `crates/ternary-lane` — the integer contract, the golden-hash
    discipline, the `pack`/`gemm`/`format` split
    ([ternary-model-lane.md](ternary-model-lane.md)).
  - `8b-is/hw-ultra` — the bare-metal memory + command-queue abstraction
    (Apple Silicon / AMD MI300X); the memory-side view of the scatter problem.
  - **MLX-QUANT** — the Apple-Silicon ternary kernel reference
    (`mlx-quant-ops`), the `{ -1, 0, +1 }` + one-scale math the C path mirrors.

The same law spans all four: **weights stay ternary/quantized the whole way,
the scale lands once, and one deterministic output is pinned by one hash.**
Project Zero is the C citizen of that lane.

*the constellation · 0 + 1 · fine touch from within · vaked.dev*
