# JuJIT Design Document

> JIT compiler for Julix. Target: peak state ASAP, no block, no long warmup.

## Core goals

1. Cold start ~0 (T0 interp, no compile)
2. Peak state fast (low threshold, fast compile, OSR)
3. No block (compile sub-ms)
4. Near-C speed for hot code

## Design decisions

### 1. Method-based vs trace-based

| | Trace-based (LuaJIT) | Method-based (V8 Sparkplug) |
|---|---|---|
| Compile unit | 1 loop iteration (trace) | whole function |
| Compile time | sub-us (trace small) | sub-ms (function) |
| Peak speed | fast (only hot path) | slightly slower (whole function) |
| Complexity | high (trace recording, stitching, abort) | low (compile function) |
| Non-loop code | bad (trace = loop only) | good (any function) |
| OSR | natural (trace = loop body) | explicit (loop header entry) |
| Code size | blowup (many traces) | compact (1 per function) |
| Deopt | complex (trace exit, side trace) | simple (interpreter resume) |

Choice: **method-based**. Julix is small, simplicity wins. tune threshold low + aggressive OSR to match trace-based peak speed. trace complexity not worth it for a small language.

### 2. Value representation

| | NaN-boxing | Pointer tagging | Fat enum (current) |
|---|---|---|---|
| Size | 64-bit | 64-bit | 16+ bytes |
| Float | inline (free) | boxed (alloc) | inline |
| Int | i32 inline, i64 boxed | i63 inline | inline |
| GC scan | tag check (1 cmp) | bit test (1 and) | match enum (slow) |
| Tag check | 1-2 instructions | 1 instruction | match (branch) |
| Fits in register | yes | yes | no |

Choice: **NaN-boxing**. Julix has first-class Float, numeric code benefits from inline doubles. i32 inline covers 99% of ints. GC scan is one comparison.

```
63                       48 47                        0
+-------------------------+----------------------------+
|  16-bit tag (NaN space) |  48-bit payload            |
+-------------------------+----------------------------+

TAG_DBL  = 0x0000          bits ARE the double (not NaN pattern)
TAG_INT  = 0xFFFC          payload low 32 = i32 (Smi)
TAG_PTR  = 0xFFFE          payload low 47 = heap pointer (8-aligned)
TAG_BOOL = 0xFFFD          payload[0] = 0/1
TAG_NULL = 0xFFFB          payload = 0
TAG_ERR  = 0xFFFA          payload = pointer to Error
```

Tag check:
```asm
; is_int?
cmp rax, 0xFFFC_0000_0000_0000
; is_ptr?
and rbx, rax, 0xFFFF_0000_0000_0000
cmp rbx, 0xFFFE_0000_0000_0000
; is_double? (NaN pattern check)
; doubles don't have tag bits, just check if not tagged
```

### 3. Compilation model

| | Foreground (block) | Background (thread) | Hybrid |
|---|---|---|---|
| Block program | yes (sub-ms) | no | minimal |
| Complexity | low | high (sync, locking) | medium |
| Memory | low | thread stack | low |
| Debug | easy | hard | medium |

Choice: **foreground**. Trace/function compile is sub-ms, not noticeable. Background adds sync complexity for no real benefit. LuaJIT compiles foreground.

### 4. Deopt strategy

| | Eager (immediate) | Lazy (safepoint) |
|---|---|---|
| Latency | 0 (jump to deopt stub) | next safepoint |
| Complexity | low | medium |
| Correctness | simple | needs safepoint tracking |
| Perf impact | guard always checks | guard sets flag, check later |

Choice: **eager** for T1 and T2. Guard fails -> immediate deopt. Simple, correct. Lazy deopt only matters for complex inlining (defer).

### 5. Hot code detection

| | Counter (per loop/call) | Sampling (time-based) | Hybrid |
|---|---|---|---|
| Overhead | 1 increment per backedge | sampling interrupt | both |
| Accuracy | exact counts | approximate | exact + time |
| Simplicity | high | medium | low |
| Catches loops | yes | yes | yes |
| Catches long functions | no (only entry) | yes (sampling) | yes |

Choice: **counter**. Simple, low overhead, catches loops (main hot code). Sampling deferred (needed for long non-loop functions, rare in Julix).

Thresholds:
- T0 -> T1: HOT_LOOP=30, HOT_CALL=50 (very low, like LuaJIT)
- T1 -> T2: 200 hits + stable type feedback (3+ hits same type)

### 6. OSR mechanism

OSR = switch tier while function is on stack.

Entry (T0 -> JIT at loop header):
1. Backedge counter crosses threshold
2. Compile function with extra entry at loop header
3. Next backedge: copy live values from interp frame to JIT frame
4. Jump into JIT loop

Exit (JIT -> T0 on deopt):
1. Guard fails in JIT code
2. Call `jujit_deopt(live_values, resume_ip)` native handler
3. Handler rebuilds interp frame from live values
4. Continue interp at resume_ip

Frame layout identical (interp-compatible) makes OSR trivial: copy slots, jump.

### 7. Cranelift version

Pin all cranelift crates to **0.135** (verified Aug 2026):
```toml
cranelift = "0.135"
cranelift-module = "0.135"
cranelift-jit = "0.135"
cranelift-native = "0.135"
```

API notes (0.135 breaking changes):
- `jump(block, &[BlockArg::Value(v)])` not `&[v]`
- `brif(cond, then, &[BlockArg::Value(t)], else, &[])`
- `fb.finalize(module.target_config())` not `fb.finalize()`
- `create_sized_stack_slot(StackSlotData { ... })` not `create_stack_slot`

## Architecture: 3-tier

```
source (.jlx) -> Lixer -> bytecode (.jlxr) -> LixVM
                                                |
                                    hot code detected (counter)
                                                |
                                    +-----------+-----------+
                                    |                       |
                              T1 baseline JIT          T2 optimizing JIT
                              (cranelift, no opt)     (type spec, inline, deopt)
                                    |                       |
                                    +-----------+-----------+
                                                |
                                          peak state: near-C
                                                |
                                          deopt -> T0 (re-tier)
```

## Phase 0: pre-JIT refactor (BLOCKER)

Current bytecode is not JIT-amenable. Must refactor before writing JIT.

### 1. Numeric slots

```
LoadVar(String) -> LoadSlot(u16)
StoreVar(String) -> StoreSlot(u16)
```

Compiler assigns each variable a per-function slot index. FunctionDef carries slot_count. JIT emits `mov rax, [fp - (slot+1)*8]` instead of hashmap lookup.

### 2. Function IDs

```
Call(String, argc) -> Call(FuncId(u32), argc)
MethodCall(String, argc) -> MethodCall(FuncId(u32), argc)
```

Name -> FuncId resolved at compile time. JIT calls direct, no string lookup.

### 3. Constant pool

Pull LoadStr/LoadFloat/LoadBytes literals out of instruction stream into `pool: Vec<Const>`, referenced by index. Compact bytecode, faster decode.

### 4. Loop headers

Emit `LoopHeader` marker or precompute backedge targets. OSR needs to know where loop headers are.

### 5. Contiguous value stack

Replace `Vec<Value>` with fixed-capacity `[Value; N]` + stack pointer. No per-op clone. Pass by copy of 64-bit word.

### 6. Error model

Replace `exit(1)` with recoverable error. Deopt needs to recover, not abort.

### 7. Shapes for Object

Replace `Object(String, HashMap<String, Value>)` with `Object(ShapeId, Vec<Value>)` (flat array, shape-computed offsets). GetField becomes array index, not hashmap lookup.

Shape transition: adding field creates new ShapeId. Object stores (ShapeId, Vec<Value>). Field offset computed from shape. Inline cache caches (ShapeId, offset).

### 8. NaN-boxed Value

Replace fat enum (16+ bytes) with 64-bit NaN-boxed word (see design decision 2).

## T0: bytecode interpreter (current + profiling)

LixVM runs bytecode. Add profiling counters:
- Backedge counter (per loop): increment on backward Jump
- Entry counter (per function): increment on Call
- Type recorder: record type tag at each op (for T2)

## T1: baseline JIT

### Strategy: Sparkplug clone

1. 1:1 bytecode -> cranelift IR, no optimization
2. Every type-dependent op calls runtime helper (jujit_add, jujit_getfield)
3. Frame layout identical to interpreter (OSR/deopt trivial)
4. Value = NaN-boxed 64-bit word

### Translation: stack bytecode -> cranelift SSA

Maintain compile-time value stack `Vec<cranelift::Value>`:

```rust
LoadInt(n) -> push iconst(I64, n)
Add        -> pop r, pop l, call jujit_add(l, r), push result
Jump(t)    -> jump blocks[t], pass vstack as block params
Return     -> pop v, return_(&[v])
```

Pre-create all blocks from bytecode CFG (ip -> Block map). Seal after all predecessors known.

### Runtime helpers (symbol table)

```rust
extern "C" fn jujit_add(l: u64, r: u64) -> u64 { ... }
extern "C" fn jujit_getfield(obj: u64, field: u64) -> u64 { ... }
extern "C" fn jujit_alloc(size: u64, kind: u64) -> u64 { ... }
extern "C" fn jujit_deopt(live: *mut u64, count: u64, ip: u64) -> ! { ... }
```

Register via `JITBuilder::symbol("jujit_add", jujit_add as *const u8)`.

### Frame layout (interpreter-compatible)

```
high addresses
+----------------------+
| caller frame         |
+----------------------+
| return address       |
+----------------------+
| saved fp             |
| function id (u32)    |
| bytecode offset slot |
| value stack region   |
|   slot[0]            |
|   slot[1]            |
|   ...                |
+----------------------+
low addresses
```

LoadSlot(i) -> `mov rax, [fp - (i+1)*8]` (single load)
StoreSlot(i) -> `mov [fp - (i+1)*8], rax`

## T2: optimizing JIT

### Type speculation

Read type feedback from T1 inline caches:
- "Add only seen Int+Int" -> emit `iadd` + guard both are Int-tagged
- "GetField(.x) only seen Point" -> emit direct load at fixed offset + shape guard

Guard fails -> deopt to T0.

### Inline caching

| State | T2 strategy |
|---|---|
| Uninitialized | call generic helper, write cache |
| Monomorphic | inline fast path + shape guard |
| Polymorphic (2-4) | small switch over shapes |
| Megamorphic (>4) | hash lookup, stop recording |

IC targets: GetField, MethodCall, IndexGet, Call.

### Inlining

- Inline small functions (<30 ops) when monomorphic
- Build callee IR into caller FunctionBuilder (no call overhead)
- Guard with inlining budget, never inline recursion

### Escape analysis

Construct whose result only used by local GetField/SetField -> scalar replacement, no allocation.

### Deopt (eager)

1. Guard fails in JIT code
2. Jump to deopt stub
3. Deopt descriptor: `{ bytecode_ip, list of (value source: reg N or frame slot K) }`
4. Reconstruct interp frame from live values
5. Continue T0 at bytecode_ip

## Code cache

```
~/.julix/cache/
+-- <hash>.jlxr       LixVM bytecode
+-- <hash>.jujit1     T1 native code
+-- <hash>.jujit2     T2 native code
```

Cache key: `hash(julix_version, cranelift_version, host_triple, source_hash, function_id, tier)`.

Never reuse code across cranelift major versions.

## JITModule limitations

`cranelift-jit` JITModule has no per-function invalidate. Only `free_memory(self)` (all-or-nothing).

Solutions:
1. One module per recompilation batch, epoch-based reclamation
2. Trampoline + version flag: entry stub jumps through global pointer, swap pointer = swap code
3. Old code leaks until no frame references it (epoch reclamation)

## GC interface (design now, implement Phase 3)

JIT must provide:
- Safepoints at every allocation + loop backedge
- Stack maps: which slots/registers hold pointers
- Write barrier call sites (behind flag, no-op for v1)

Without stack maps, GC is conservative (leaks). Precise GC requires these maps.

## Implementation phases

| Phase | Content | Target |
|---|---|---|
| 0 | Pre-JIT refactor (slots, IDs, NaN-box, shapes, stack, error model) | 8-20x speedup from refactor alone |
| 1 | T1 baseline JIT (cranelift, 1:1, runtime helpers, OSR entry) | 15-30x vs original |
| 2 | Inline caching + type feedback | feed T2 |
| 3 | T2 optimizing JIT (type spec, inline, deopt, escape analysis) | 40-100x on hot numeric |
| 4 | GC stack maps + code cache | precise GC, persistent cache |

## Performance expectations

| Stage | Speedup vs current | Notes |
|---|---|---|
| Phase 0 refactor | 8-20x | most gain from removing string lookup + clone |
| + T1 baseline | 15-30x | cranelift codegen, no dispatch loop |
| + T2 optimizing | 40-100x | type spec + inline on hot code |
| cranelift vs LLVM | 80-90% of LLVM | acceptable trade for fast compile |

Compile time:
- T1: sub-ms to few ms per function
- T2: 5-50 ms per function (with inlining)

## Target: peak state ASAP

```
open file -> 0ms (T0 interp, no compile)
  |
  run ~30 loop iterations or ~50 calls
  |
  T1 compile hot code (sub-ms per function)
  |
  OSR: switch to native mid-loop (~0)
  |
  run ~200 iterations with type feedback
  |
  T2 compile optimized (5-50ms per function)
  |
  OSR: switch to optimized native (~0)
  |
  peak state: near-C speed
```

User experience: open file, runs, fast. No visible warmup. Like LuaJIT, not V8.

## Pitfalls

| Pitfall | Avoidance |
|---|---|
| JIT before refactor | Phase 0 is non-negotiable |
| Mismatched JIT vs interp frame | identical frame layout from day 1 |
| Mixed cranelift versions | pin all to same version |
| Vec<u8> for executable code | dedicated mmap arena with PROT_EXEC |
| macOS arm64 W^X | MAP_JIT + pthread_jit_write_protect_np |
| exit(1) in interpreter | replace with recoverable error for deopt |
| Unboxing in T1 | don't, T1 keeps boxed, unboxing is T2 |
| Over-inlining | budget cap, never inline recursion |
| Stale type feedback cache | start without caching feedback |
| JITModule no per-function free | epoch-based reclamation |
| BlockArg::Value wrap | 0.135 API, must wrap values in jump/brif |
| finalize needs target_config | 0.135 API, not no-arg anymore |
