# JuJIT Design Document

> JIT compiler for Julix. Target: peak state ASAP, no block, no long warmup.

## Core goals

1. Cold start ~0 (T0 interp, no compile)
2. Peak state fast (low threshold, fast compile, OSR)
3. No block (compile sub-ms or background)
4. Near-C speed for hot code

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

### 8. NaN-boxed Value

Replace fat enum (16+ bytes) with 64-bit NaN-boxed word:

```
63                       48 47                        0
+-------------------------+----------------------------+
|  16-bit tag (NaN space) |  48-bit payload            |
+-------------------------+----------------------------+

TAG_DBL  = 0x0000          -> bits ARE the double
TAG_INT  = 0xFFFC          -> payload low 32 = i32 (Smi)
TAG_PTR  = 0xFFFE          -> payload low 47 = heap pointer
TAG_BOOL = 0xFFFD          -> payload[0] = 0/1
TAG_NULL = 0xFFFB          -> payload = 0
TAG_ERR  = 0xFFFA          -> payload = pointer to Error
```

Floats inline (no alloc). Ints up to i32 inline. Large i64 boxed as HeapInt. GC scans: tag == TAG_PTR || tag == TAG_ERR.

## T0: bytecode interpreter (current)

LixVM runs bytecode. Add profiling counters:
- Backedge counter (per loop): increment on backward Jump
- Entry counter (per function): increment on Call

Thresholds:
- T0 -> T1: HOT_LOOP=60, HOT_CALL=100 (low, like Sparkplug)
- T1 -> T2: ~1000 hits + stable type feedback

## T1: baseline JIT

### Strategy: Sparkplug clone

1. 1:1 bytecode -> cranelift IR, no optimization
2. Every type-dependent op calls runtime helper (jujit_add, jujit_getfield)
3. Frame layout identical to interpreter (OSR/deopt trivial)
4. Value = boxed pointer (*mut Value as i64) in T1, NaN-box in T2

### Cranelift setup

```toml
[dependencies]
cranelift = "0.135"
cranelift-module = "0.135"
cranelift-jit = "0.135"
cranelift-native = "0.135"
```

All cranelift crates must be same version (they move in lockstep).

### API notes (0.135 breaking changes)

- `jump(block, &[BlockArg::Value(v)])` not `&[v]`
- `brif(cond, then, &[BlockArg::Value(t)], else, &[])`
- `fb.finalize(module.target_config())` not `fb.finalize()`
- `create_sized_stack_slot(StackSlotData { ... })` not `create_stack_slot`

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
extern "C" fn jujit_add(l: i64, r: i64) -> i64 { ... }
extern "C" fn jujit_getfield(obj: i64, field: i64) -> i64 { ... }
extern "C" fn jujit_alloc(size: i64, kind: i64) -> i64 { ... }
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

### Deopt

Eager deopt (v1): guard fails -> jump to deopt stub -> reconstruct interpreter state -> continue T0.

Deopt descriptor per deopt point: `{ bytecode_ip, list of (value source: reg N or frame slot K) }`.

## OSR (On-Stack Replacement)

### OSR entry (T0 -> JIT at loop header)

1. Loop counter crosses threshold mid-loop
2. Compile function with extra entry at loop header
3. Copy live interpreter state into JIT frame
4. Jump into compiled loop

### OSR exit (deopt)

1. Guard fails in JIT code
2. Call `jujit_deopt(live_values, resume_ip)` (native)
3. Native handler rebuilds interpreter frame
4. Continue T0

Cranelift has no built-in OSR. JuJIT implements via trap -> native handler -> interpreter.

## Code cache

```
~/.julix/cache/
+-- <hash>.jlxr       LixVM bytecode
+-- <hash>.jujit1     T1 native code
+-- <hash>.jujit2     T2 native code
```

Cache key: `hash(julix_version, cranelift_version, host_triple, source_hash, function_id, tier)`.

Never reuse code across cranelift major versions.

## GC interface (design now, implement Phase 3)

JIT must provide:
- Safepoints at every allocation + loop backedge
- Stack maps: which slots/registers hold pointers
- Write barrier call sites (behind flag, no-op for v1)

Without stack maps, GC is conservative (leaks). Precise GC requires these maps.

## JITModule limitations

`cranelift-jit` JITModule has no per-function invalidate. Only `free_memory(self)` (all-or-nothing).

Solutions:
1. One module per recompilation batch, epoch-based reclamation
2. Trampoline + version flag: entry stub jumps through global pointer, swap pointer = swap code
3. Old code leaks until no frame references it (epoch reclamation)

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

## Target: peak state ASAP

```
open file -> 0ms (T0 interp, no compile)
  |
  run ~60 loop iterations or ~100 calls
  |
  T1 compile hot code (sub-ms per function)
  |
  OSR: switch to native mid-loop (~0)
  |
  run ~1000 iterations with type feedback
  |
  T2 compile optimized (5-50ms per function)
  |
  OSR: switch to optimized native (~0)
  |
  peak state: near-C speed
```

User experience: open file, runs, fast. No visible warmup. Like LuaJIT, not V8.
