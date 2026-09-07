# Julix Syntax v0.0.2

> Dynamic, script-first, GC-managed. Nhanh hơn Python, sâu hơn JS/Go để làm system. User không phải nghĩ memory.

## 1. Entry point

Top-level là entry. Không có `main()`.

```julix
println("hello from julix");
```

## 2. Literals

| Loại | Cú pháp | Ví dụ |
|---|---|---|
| Int | decimal, hex, oct, bin, underscore | `42`, `0xFF`, `0o17`, `0b1010`, `1_000_000` |
| Float | có dấu chấm | `3.14`, `1.0` |
| Bool | `true` / `false` | |
| String | `"..."` | `"hello\n"` |
| Bytes | `b"..."` | `b"GET / HTTP/1.1\r\n"` |
| Null | `null` | |
| F-string | `f"..."` | `f"{x} = {y}"` |

Runtime tự chọn width (i32/i64/u64/f64). User chỉ gõ `int`, `float`, không thấy `i32`/`i64`/`u8`/`f32`.

## 3. Variable

```julix
const pi = 3.14;        // bất biến, gán 1 lần
let count = 0;          // khả biến, gán lại được
count = count + 1;
count++;
count += 1;

let big: int = 1_000_000;   // type optional
```

| Keyword | Ý nghĩa |
|---|---|
| `const` | bất biến |
| `let` | khả biến |

Compound assign: `+= -= *= /= %=`
Increment/decrement: `++ --` (prefix + postfix)

## 4. Operators

```julix
// số học
a + b;
a - b;
a * b;
a / b;      // int/int = int (truncate), có float = float
a % b;

// so sánh
a == b;
a != b;
a < b;
a > b;
a <= b;
a >= b;

// logic (2 kiểu, chọn 1)
a && b;
a || b;
!a;
// hoặc
a and b;
a or b;
not a;

// bit
a & b;
a | b;
a ^ b;
a << b;
a >> b;
```

Strict bool. Không truthy/falsy. Không ép kiểu implicit.

```julix
if (1) { }          // error: 1 là int, không phải bool
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

- `()` bắt buộc cho condition
- `elif` thay `else if`
- `0..10` = range [0, 10), exclusive
- `{}` optional cho single statement

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
- Param type optional (`: string`), infer mặc định
- Return type optional (`: int`), infer mặc định
- `return` để trả, implicit `null` nếu không return
- `;` bắt buộc
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
// hết scope -> deinit chạy -> close(fd)
```

- `type` keyword, K&R braces
- Field type bắt buộc (`x: int`)
- Method: `function name(self, ...) { ... }`
- `deinit` là hook dọn resource, GC trigger, user không phải nghĩ
- Constructor: `Type(field: value)`, không `new`
- Copy on assign, không ownership/move
- Memory do GC lo

## 8. Print

```julix
print("no newline");
println("with newline");
println(f"{x} = {y}");
```

- `print()` không newline
- `println()` có newline
- `f"..."` f-string, `{expr}` chèn giá trị

## 9. Comment

```julix
// comment 1 dòng
// không slop, chỉ nêu constraint / gotcha / FIXME
```

- `//` only
- Không `///`, không `//!`, không `#`, không block comment

## 10. Code style

- K&R braces
- `;` bắt buộc
- Không em-dash trong code/doc/commit
- Comment `//` only, ngắn gọn, dễ hiểu, không slop
- Tên hàm/biến tự nói thay comment

## 11. Còn thiếu (sau này)

- list/map literal + index
- error handling
- FFI / syscall / buffer / raw ptr
- module / import
- closure / lambda
- async / coroutines
- enum / pattern matching
- string methods
