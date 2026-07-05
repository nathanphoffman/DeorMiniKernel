# Best Practices
Style recommendations not enforced by the transpiler.

---
## Order of Declaration
1. Imports - Everything else could use it, relies on nothing else in the file
2. Enums - Relies likely on nothing else in the file
3. Structs - Reliant on most everything above but still structural (so above functions)
4. Types - Type validators being types must be defined early
5. Shapes - Shapes can reference almost anything above
6. Macros
7. Functions - Reliant on everything above

---
## Spacing
Add a blank line before `return` in any function body that contains more than one statement. One-liner functions (single expression, no bindings) are exempt. Try to keep blocks spaced out keeping like concepts with one another.

Functions get extra separation from everything around them: leave 2 blank lines (3 returns) between a function and the next function or block. All other top-level blocks (structs, enums, shapes, macros) only need 1 blank line between them.

**Recommended:**
```deor
shape rollResultList = list of RollResult

fn int sum_rolls(rollResultList rolls)
    # is its own thing, just printing new line after these 2 lines
    roll_notice as "Going to roll"
    print(roll_notice)

    # belongs together as it is all part of the loop
    sum as 0
    for roll in rolls
        (value) in roll
        val as 0
        if value is valid
            val = (avow value)
        sum = sum + val

    return sum
```

**Exempt — one-liner, no blank line needed:**
```deor
fn int square(int val)
    val * val
```
---
## Multi-line Calls — No Trailing Comma

When a call wraps across multiple lines, don't put a comma after the last argument. Deor favors reading like a plain human list — nobody writes "lettuce, tomato, onion," with a trailing comma, so don't write one here either.

```deor
Connection conn = Connect(
    host,
    port,
    timeout
)
```

---
## Construction and Destructuring

Field order does not matter — all construction and destructuring forms are name-matched. Any subset in any order is valid for destructuring; fields in construction are matched by variable name to struct field name.

```deor
struct Employee
    int employee_id
    string first_name
    string last_name

(employee_id, first_name, last_name) in employee
Employee emp = (employee_id, first_name, last_name)
emp as (employee_id, first_name, last_name)
```

Even though order is not enforced, write fields in the same order they appear in the struct declaration. It makes construction and destructuring sites easier to scan and keeps things consistent across the codebase.

Additionally, all `in` extractions should appear before any logic (assignments, expressions, control flow) within their block. Applies to function bodies, loop bodies, and if/else bodies.

**Correct:**
```deor
fn RollResult roll_die(Die die)

    (sides, label) in die

    min as 1
    int raw = m_rand_int(min, sides)

    Roll value = raw
    string source = label
    RollResult result = (value, source)

    return result
```
---
## avow

Always capture `avow` into its own binding — do not use `(avow val)` inline inside a larger expression.

**Recommended:**
```deor
if result is valid
    int val = (avow result)
    total = total + val
```

**Avoid:**
```deor
if result is valid
    total = total + (avow result)
```

Also always guard `avow` with an `is valid` check immediately above it, or add a comment explaining why the value is guaranteed valid at that point.

---
## Avoid Deep Nesting

Limit nesting to two or three levels. Deeply nested code is hard to read and usually signals that logic should be extracted into a helper function. Prefer early returns and guard clauses over deep `if/else` trees.

**Preferred — early return flattens nesting:**
```deor
fn string classify(int val)
    if val < 0
        return "negative"
    if val is 0
        return "zero"
    return "positive"
```

**Avoid — deep nesting:**
```deor
fn string classify(int val)
    if val < 0
        return "negative"
    else
        if val is 0
            return "zero"
        else
            return "positive"
```

---
## Keep Functions Small

A function should do one thing. If a function body is growing long or handles multiple distinct concerns, extract the inner logic into named helper functions. The 3-parameter limit already encourages this — if you need more context, you should already be reaching for a struct.

---
## Functions vs. Macros — Performance in Loops

Prefer functions for reusable logic that is called outside loops. Inside a loop body, prefer macros when the operation is simple and call overhead matters — macros expand inline, whereas function calls add a frame per iteration.

**Use a function — called once or outside a loop:**
```deor
fn int square(int val)
    val * val

int area = square(side)
```

**Use a macro — called inside a loop:**
```deor
macro sq(val)
    val * val

for item in items
    int area = sq(val)
```

If the logic is non-trivial, extract it into a function regardless — readability wins over minor call overhead for complex operations.

---
## Reusable Consts — via Macros

Deor has no global scope, so a `const` can't be declared once and shared across every function that needs it — it only exists inside the block where it's declared. When the same constant values are needed in multiple functions, declare them once inside a `macro` and `macro_run` it wherever they're needed. Because the macro body is inlined at each call site, every function gets its own copy of the same named consts — nothing is shared at runtime, but the names and values stay consistent everywhere they're used.

```deor
macro use_log_consts
    const string INFO_PREFIX = "[INFO] "
    const string ERROR_PREFIX = "[ERROR] "

fn void log_info(string msg)
    macro_run use_log_consts
    print(INFO_PREFIX + msg)

fn void log_error(string msg)
    macro_run use_log_consts
    print(ERROR_PREFIX + msg)
```

Don't `macro_run` the same const-macro twice in one function body — the second inlining redeclares the same variable names in the same scope, which the transpiler always rejects. This is a separate check from [top-level duplicate declarations](enforced_practices.md#duplicate-top-level-names) — variable redeclaration inside a function body is never allowed, regardless of the `ENFORCE_UNIQUE_*` pragmas.

---
## File Length

Keep files to a reasonable length. There is no hard limit, but when a file starts to feel long, consider splitting it. A natural split point is when the file contains multiple distinct concerns — for example, separate structs and their associated functions into their own files.

---
## Prefer `is not` Over `not ... is`

When negating a comparison, use `is not` rather than wrapping the comparison in `not`. `val is not 5` is required for bare identifiers (the transpiler rejects `not val is 5`), and the same preference applies even in the cases the transpiler doesn't catch, like `not (a > b) is y` or `not some_func() is y` — reorder these to `(a > b) is not y` and `some_func() is not y` instead.

**Recommended:**
```deor
if (a > b) is not y
    ...
```

**Avoid:**
```deor
if not (a > b) is y
    ...
```

Reach for plain `not` only when there's no comparison to reorder around — negating a standalone boolean value or expression:

```deor
if not done
    ...

opposite as not original
```

---
## Naming External Libs
Because Deor does not support third-party importing, the standard convention is to copy Deor files into the `lib/` folder, prefixed the same two-letter way as the standard library. See [Libs — Naming Convention](docs/libs.md#naming-convention) for the full prefix scheme and examples.