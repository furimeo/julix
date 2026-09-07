# Julix Syntax v0.0.2

> Dynamic, script-first, GC-managed. Faster than Python, deeper than JS/Go for system work. The user does not think about memory.

## 1. Entry point

Top-level is the entry. No `main()`.

```julix
println("hello from julix");
```

## 2. Literals

| Type | Syntax | Example |
|---|---|---|
| Int | decimal, hex, oct, bin, underscore | `42`, `0xFF`, `0o17`, `0b1010`, `1_000_000` |
| Float | with decimal point | `3.14`, `1.0` |
| Bool | `true` / `false` | |
| String | `"..."` | `"hello\n"` |
| Bytes | `b"..."` | `b"GET / HTTP/1.1\r\n"` |
| Null | `null` | |
| F-string | `f"..."` | `f"{x} = {y}"` |

The runtime picks the width (i32/i64/u64/f64). The user types `int`, `float`, and never sees `i32`/`i64`/`u8`/`f32`.

## 3. Variable

```julix
const pi = 3.14;        // immutable, assigned once
let count = 0;          // mutable, can reassign
count = count + 1;
count++;
count += 1;

let big: int = 1_000_000;   // type optional
```

| Keyword | Meaning |
|---|---|
| `const` | immutable |
| `let` | mutable |

Compound assign: `+= -= *= /= %=`
Increment/decrement: `++ --` (prefix and postfix)

## 4. Operators

```julix
// arithmetic
a + b;
a - b;
a * b;
a / b;      // int / int = int (truncated), mixed with float = float
a % b;

// comparison
a == b;
a != b;
a < b;
a > b;
a <= b;
a >= b;

// logic (two styles, pick one)
a && b;
a || b;
!a;
// or
a and b;
a or b;
not a;

// bitwise
a & b;
a | b;
a ^ b;
a << b;
a >> b;
```

Strict bool. No truthy/falsy. No implicit coercion.

```julix
if (1) { }          // error: 1 is int, not bool
if (x != 0) { }    // OK
```

## 5. Control flow

```julix
if (cond) {
    ...
} elif (cond2) {
    ...
} else {
    ...
}

while (cond) {
    ...
}

for (i in 0..10) {
    print(i);
}

for (x in [1, 2, 3]) {
    print(x);
}

break;
continue;
```

- `()` required around the condition
- `elif` instead of `else if`
- `0..10` = range [0, 10), exclusive
- `{}` optional for a single statement

```julix
if (n < 2) return n;

while (true) break;
```

## 6. Function

```julix
function add(a, b) {
    return a + b;
}

function fib(n): int {
    if (n < 2) return n;
    return fib(n - 1) + fib(n - 2);
}

function greet(name: string) {
    println(f"hello {name}");
}

greet("julix");
print(fib(10));
```

- `function` keyword, K&R braces
- Parameter type optional (`: string`), inferred by default
- Return type optional (`: int`), inferred by default
- `return` to return, implicit `null` if missing
- `;` required
- Recursion OK

## 7. Type

```julix
type Point {
    x: int;
    y: int;

    function add(self, other): Point {
        return Point(x: self.x + other.x, y: self.y + other.y);
    }
}

type Socket {
    fd: int;

    function deinit(self) {
        close(self.fd);
    }

    function write(self, buf) {
        send(self.fd, buf, len(buf));
    }
}

let p = Point(x: 1, y: 2);
print(p.x);

let s = Socket(fd: 3);
s.write(b"hello");
// out of scope -> deinit runs -> close(fd)
```

- `type` keyword, K&R braces
- Field type required (`x: int`)
- Method: `function name(self, ...) { ... }`
- `deinit` is a resource cleanup hook, triggered by GC, user does not think about it
- Constructor: `Type(field: value)`, no `new`
- Copy on assign, no ownership/move
- Memory is handled by GC

## 8. Print

```julix
print("no newline");
println("with newline");
println(f"{x} = {y}");
```

- `print()` no newline
- `println()` with newline
- `f"..."` f-string, `{expr}` inserts a value

## 9. Comment

```julix
// single line comment
// no slop, only state constraints / gotchas / FIXMEs
```

- `//` only
- No `///`, no `//!`, no `#`, no block comments

## 10. Code style

- K&R braces
- `;` required
- No em-dash in code, docs, or commits
- Comments `//` only, short, clear, no slop
- Function and variable names speak for themselves

## 11. Not yet defined (later)

- list/map literal and indexing
- error handling
- FFI / syscall / buffer / raw pointer
- module / import
- closure / lambda
- async / coroutines
- enum / pattern matching
- string methods
