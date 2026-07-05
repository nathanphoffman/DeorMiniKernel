<!-- title: Deor Specification -->
<!-- [Deor Specification Index](index.md) -->
<!-- themes: dusk -->
# Loops

## Collection Iteration
```deor
for room in rooms
    (name) in room
    (occupied) in room
    if occupied
        print(name)
```

---
## Move Iteration
Plain `for item in collection` borrows the collection (`for item in &collection`) — `item` is a reference. `for move (item in collection)` consumes the collection instead — `item` is owned, and `collection` cannot be used after the loop.

```deor
for move (item in collection)
    process(item)
```

```rust
for item in collection {
    process(item);
}
```

Use this when the loop body needs to actually own each item (e.g. pass it into something that takes ownership, or store it elsewhere) — it avoids having to clone each borrowed item yourself. See [Move](docs/move.md#loop-iteration).

---
## Numeric Iteration
`range(count)` produces values from `0` to `count - 1`. `range(a_start_num, an_end_num)` produces values from `a_start_num` up to but not including `an_end_num`. `range` is a built-in, so literals are valid directly.

```deor
for idx in range(count)
    ...

start as 1
stop as 11
for idx in range(start, stop)
    print(idx)    # prints 1 through 10
```

```rust
for idx in 0..count {
    ...
}
for idx in 1..11 {
    println!("{}", idx);
}
```

**Note:** `range` with two bounds requires both to be named variables — the same rule as any multi-argument call. `end` is a reserved keyword; the conventional names for range bounds are `start` and `stop`.

`range(count)` is shorthand for `range(0, count)`.

---
## Bare Tuple Range
`(start, end)` is an alternative to `range(start, end)` when both bounds are already named variables in scope. Produces identical Rust output — use whichever reads more clearly.

```deor
for idx in (low, high)
    print(idx)
```

```rust
for idx in low..high {
    println!("{}", idx);
}
```

The index-free form works too:

```deor
for in (low, high)
    do_something()
```

```rust
for _ in low..high {
    do_something();
}
```

---
## Repeat Without an Index
When the loop index is not needed, write `for in range(n)` — the variable name is omitted but `in` stays:

```deor
for in range(10)
    do_something()

start as 1
stop as 11
for in range(start, stop)
    do_something()
```

```rust
for _ in 0..10 {
    do_something();
}
for _ in 1..11 {
    do_something();
}
```

---
## `break` — Exit a Loop Early
`break` exits the innermost loop immediately. Execution continues after the loop body.

Deor:
```deor
found as false
for room in rooms
    (occupied) in room
    if occupied
        found = true
        break
```

Rust:
```rust
let mut found = false;
for room in &rooms {
    let occupied = room.occupied;
    if occupied {
        found = true;
        break;
    }
}
```
`break` applies to the **innermost** loop only. Labeled breaks (breaking out of an outer loop from an inner one) are not supported.

---
## Condition-Based Loops — `for if`
`for if condition` is Deor's while loop. It loops as long as the condition is true.

```deor
for if cur < token_count
    # process token at cur
    cur = cur + 1
```

```rust
while cur < token_count {
    // process token at cur
    cur = cur + 1;
}
```

`for if true` is the infinite loop form — use with `break` to exit:

```deor
for if true
    if done
        break
    do_work()
```

```rust
loop {
    if done { break; }
    do_work();
}
```

`for if true` specifically (the literal condition `true`, not an expression that merely evaluates true) generates a bare `loop { ... }` rather than `while true { ... }`. This matters for functions that return a value: Rust gives `loop` the special "never returns normally" type when it has no reachable `break`, which lets the compiler accept a function whose only return path is inside the loop. `while true { ... }`, even though it runs identically, doesn't get that treatment from Rust — a function ending in `while true { ... }` with no value-producing code after it fails to compile with "expected T, found ()", regardless of whether every branch inside actually returns. Any other condition (`for if cur < token_count`, `for if not done`, etc.) always generates `while`, unaffected by this.

```deor
fn int ask_until_valid()
    for if true
        int val = read_input()
        if is_valid(val)
            return val
```

```rust
fn ask_until_valid() -> i64 {
    loop {
        let val: i64 = read_input();
        if is_valid(val) {
            return val;
        }
    }
}
```

`for if` fits the same keyword as collection and range iteration — `for` is Deor's single loop keyword, and the token after it determines the form: `item in collection`, `in range(n)`, or `if condition`.

---
## `continue` — Skip to Next Iteration

`continue` skips the rest of the current loop body and moves to the next iteration.

```deor
for item in items
    (active) in item
    if not active
        continue
    process(item)
```

```rust
for item in &items {
    let active = item.active;
    if !active {
        continue;
    }
    process(item);
}
```

Like `break`, `continue` applies to the innermost loop only.
