# Julix Versioning

Julix has three independent versions, each tracking a different component.

## 1. Julix version

The language version: syntax, standard library, and semantics of what the user writes.

Example: `Julix 0.0.2`

Changes when:
- syntax is added or modified
- standard library is added or modified
- compatibility is broken

## 2. LixVM version

The runtime version: interpreter, garbage collector, memory model, and runtime internals.

Example: `LixVM 0.0.1`

Changes when:
- interpreter is modified
- garbage collector is modified
- memory model is modified
- runtime internals change

Independent of the Julix version. A single LixVM can run multiple Julix versions.

## 3. JuJIT version

The JIT compiler version: codegen, optimization passes, and tier management.

Example: `JuJIT 0.0.0` (not yet built, phase 2)

Changes when:
- a JIT tier is added or modified
- codegen is modified
- optimization is modified

Independent of the LixVM version. JuJIT is a layer on top of LixVM.

## Current status

| Component | Version | Notes |
|---|---|---|
| Julix | 0.0.4 | if/elif/else, while/for, break/continue, arithmetic, comparison, logic |
| LixVM | 0.0.3 | bytecode VM, jump patching, loop support |
| JuJIT | 0.0.0 | not built yet, phase 2 |

## Format

Semantic versioning: `MAJOR.MINOR.PATCH`

- MAJOR: breaking change
- MINOR: new feature, no breakage
- PATCH: bug fix, no breakage

## CLI

```sh
julix --version        # print all three
# Julix 0.0.2
# LixVM 0.0.0
# JuJIT 0.0.0

julix --julix-version  # only Julix
julix --lixvm-version  # only LixVM
julix --jujit-version  # only JuJIT
```
