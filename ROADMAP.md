# Julix Roadmap

## Phase 0 - Bootstrap interpreter (LixVM)

| Step | Feature | Example |
|---|---|---|
| 1 | literals (int, string) + print | `print("hello")` |
| 2 | let / const + arithmetic | `let x = 1 + 2` |
| 3 | if / elif / else | branching |
| 4 | while / for + break / continue | loops |
| 5 | function | `fib(10)` |
| 6 | type + constructor | `Point.add()` |
| 7 | deinit hook | `Socket` RAII |

## Phase 1 - Script-ready

| Step | Feature |
|---|---|
| 8 | string / bytes literal + concat |
| 9 | list + map literal, index, len |
| 10 | error handling |
| 11 | FFI / syscall binding |
| 12 | buffer / raw pointer |

## Phase 2 - JIT

| Step | Feature |
|---|---|
| 13 | inline caching |
| 14 | baseline JIT (JuJIT T1) |
| 15 | optimizing JIT (JuJIT T2) + deopt |
| 16 | escape analysis |

## Phase 3 - Memory model

| Step | Feature |
|---|---|
| 17 | arena per scope |
| 18 | generational mark-sweep GC |
| 19 | snapshot cache |

## Phase 4 - Beyond

| Step | Feature |
|---|---|
| 20 | closure / first-class function |
| 21 | module / import |
| 22 | string interp / format |
| 23 | enum / optional / pattern matching |
| 24 | async / coroutines |
| 25 | concurrency (channels / actors) |
