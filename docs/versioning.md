# Julix Versioning

Julix co 3 version doc lap, biet ro thang nao thay doi.

## 1. Julix version

Phien ban ngon ngu: cu phap, stdlib, semantic nhung gi user viet.

Vi du: `Julix 0.0.2`

Thay doi khi:
- them/sua cu phap
- them/sua stdlib
- break compatibility

## 2. LixVM version

Phien ban trinh chay code (interpreter + runtime + GC).

Vi du: `LixVM 0.0.1`

Thay doi khi:
- sua interpreter
- sua GC
- sua memory model
- sua runtime internals

Khong phu thuoc Julix version. LixVM co the chay nhieu Julix version.

## 3. JuJIT version

Phien ban trinh dich JIT.

Vi du: `JuJIT 0.0.0` (chua co, phase 2)

Thay doi khi:
- them/sua JIT tier
- sua codegen
- sua optimization

Khong phu thuoc LixVM version. JuJIT la layer tren LixVM.

## Trang thai hien tai

| Component | Version | Ghi chu |
|---|---|---|
| Julix | 0.0.2 | syntax da chot, chua co stdlib |
| LixVM | 0.0.0 | chua build |
| JuJIT | 0.0.0 | chua build, phase 2 |

## Format

Semver: `MAJOR.MINOR.PATCH`

- MAJOR: break change
- MINOR: them feature, khong break
- PATCH: fix bug, khong break

## CLI

```sh
julix --version        # in ca 3
# Julix 0.0.2
# LixVM 0.0.0
# JuJIT 0.0.0

julix --julix-version  # chi Julix
julix --lixvm-version  # chi LixVM
julix --jujit-version  # chi JuJIT
```
