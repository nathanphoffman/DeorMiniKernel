<!-- title: Deor Specification -->
<!-- [Deor Specification Index](index.md) -->
<!-- themes: dusk -->
# Enforced Practices
These rules are enforced by the transpiler. Violations produce warnings or compile-time errors.

---
## Naming Conventions
- enums, structs, and custom types (type validators) MUST be PascalCase — bold and structural, distinct from the code around it
- shapes must be camelCase — aliasing, distinct but blends in more than PascalCase
- variable and function names must be snake_case — the most common case, kept plain and readable
- `const` variable names must be SCREAMING_SNAKE_CASE — signals a fixed, never-reassigned value at a glance
---
## Minimum Name Length — 3 Characters
All identifiers must be at least 3 characters long. This applies to every named thing in Deor source: variables, function parameters, function names, struct names, validator type names, struct field names, and list names.

```deor
int val = 5      # correct
int vl = 5       # transpiler error — 2 characters
int v = 5        # transpiler error — 1 character

fn int add(int left, int right)   # correct
fn int add(int a, int b)          # transpiler error — parameters too short

type Roll(int val)    # correct
type Roll(int n)      # transpiler error — parameter too short
```

There are no exceptions. All runtime identifiers — variables, parameters, fields, functions, type names, and shape names — must be at least 3 characters.

---
## Ordering
Imports are required to be at the top of a file, however there are no other restrictions, but there are suggestions for [best practices](docs/best_practices.md).


---
## Validator Type Predicate Required

The predicate body is mandatory — see [Validator Types — Declaration](docs/validator_types.md#declaration) for why.

**Correct:**
```deor
type Positive(int val)
    val > 0
```

**Incorrect — transpiler errors:**
```deor
type Positive(int val)
```

---
## `empty` and List Initialization

List shape variables can be initialized with either `empty` or a list literal:

```deor
roomList rooms = empty              # starts empty
roomList rooms = [kitchen, office]  # starts with items
```

`empty` is not valid for validator types — declare a validator type variable without a value to start it as not valid:

```deor
Roll best                 # correct — starts not valid
Roll best = empty         # transpiler error — empty not valid for validator types
```

---
## `const` — No Reassignment

`const` variables cannot be reassigned. Attempting to assign a new value to a `const` variable after its declaration is a transpiler error.

```deor
const string PIPE = "|"
PIPE = "/"              # transpiler error — cannot reassign a const variable
```

Use a plain typed binding (`string pipe = "|"`) if you need a variable that may change.

---
## `as` — No Type Annotation

`as` is the type-inferring binding form. Pairing it with an explicit type is always a transpiler error — when you have an explicit type, use `=` instead.

```deor
count as 0          # correct — int inferred
int count as 0      # transpiler error — annotation not allowed with as
int count = 0       # correct
```

---
## Duplicate Top-Level Names

By default, declaring two top-level items with the same name is **not** an error — whichever one the importer merges in first silently wins, and the rest are dropped. This is deliberate: it's what lets the standard library have several `lib/*.deor` files each redeclare the same convenience alias (`shape stringList = list of string`, for example) so any one of them can be imported standalone.

If you want stricter checking, opt in with a pragma as the very first statement(s) of `fn void main()`:

```deor
fn void main()
    ENFORCE_UNIQUE_FILE_DECLARATIONS
    ENFORCE_UNIQUE_IMPORT_DECLARATIONS
    ...
```

- `ENFORCE_UNIQUE_FILE_DECLARATIONS` — a name declared twice **in the same file** is a hard transpiler error.
- `ENFORCE_UNIQUE_IMPORT_DECLARATIONS` — a name declared in two **different** files is a hard transpiler error.

They're independent — set one, both, or neither, in either order. Both apply across all declaration forms (`struct`, `enum`, `shape`, `type`, `fn`) and across declaration kinds, so a `struct Foo` and a `fn Foo` sharing a name are checked the same way. The check runs while imports are being merged, before any other validation, and fails fast on the first collision found.

### Why two separate pragmas, and why opt-in

This is deliberately Fortran-`IMPLICIT NONE`-flavored: loose by default, strict only if you ask for it, and the ask has to be a visible declaration at the top of the entry point rather than a buried config flag.

The two checks are kept independent rather than folded into one "strict mode" because project style varies:

- Some projects want strictness from day one and set both — accepting the cost of pulling shared aliases (like the stdlib's `stringList`/`tList` pattern) out into their own dedicated file so nothing collides and accepting responsibility to clean up built-in lib shapes, or implementing your own libs.
- Some projects are one long, run-on file by convention, with the same shape or struct deliberately redeclared in different regions for local readability — `ENFORCE_UNIQUE_IMPORT_DECLARATIONS` alone fits that style without breaking it.
- Some small projects don't care about cross-file collisions at all but still want a typo like a copy-pasted `struct Player` block in the same file to fail loudly — `ENFORCE_UNIQUE_FILE_DECLARATIONS` alone covers that.
- Some projects want to redeclare a generic library shape locally after `T`-substitution, just to see its concrete form inline — the default, with neither pragma, supports that without any opt-in at all.

Because they're separate statements rather than one combined flag, the choice a project made is visible directly at the top of `main()`, not inferred from a build setting elsewhere.

```deor
struct Room
    string name

struct Room    # error only with ENFORCE_UNIQUE_FILE_DECLARATIONS
    float area

fn void process(Room item)
    ...

fn void process(string label)    # error only with ENFORCE_UNIQUE_FILE_DECLARATIONS
    ...
```

Variable shadowing within function bodies is unrelated to either pragma and is always allowed — see below.

---
## Variable Shadowing

Variable shadowing is allowed. A new declaration with the same name in the same block or an inner block replaces the binding from that point forward.

```deor
int val = 5
int val = 10    # allowed — val is now 10
print(val)      # 10
```

Inner block shadowing is also allowed and does not affect the outer binding:

```deor
int val = 5
if condition
    int val = 10    # shadows outer val within this block only
    print(val)      # 10
print(val)          # 5
```

**Exception:** built-in function names (`print`, `crash`, `len`, `range`, `args`, `input`) can never be shadowed, even though they aren't reserved keywords — see [Syntax — Built-in Function Names](docs/syntax.md#built-in-function-names).

```deor
int print = 5    # transpiler error — print is a built-in, not shadowable
```

---
## Maximum 3 Parameters per Function

Functions may accept at most 3 parameters. If more context is needed, bundle values into a struct first. This is enforced by the transpiler.

```deor
fn roomList filter(roomList items, string query, filterFunc predicate)    # correct — 3 params
```

```deor
fn roomList filter(roomList items, string query, int limit, filterFunc predicate)    # transpiler error — 4 params
```

`func` shape parameters count toward the limit the same as data parameters.

---
## No `func` Shapes as Struct Fields

Struct fields must be data types — primitives, validator types, other structs, or list shapes. A `func` shape field would make the struct a closure in disguise, which Deor does not allow.

**Correct:**
```deor
fn roomList apply(roomList items, filterFunc predicate)    # func as parameter — fine
```

**Incorrect — transpiler error:**
```deor
struct Config
    roomList items
    filterFunc predicate    # func shape as struct field — not allowed
```

---
## Unified `()` Rule — Named Variables

Everything placed inside `()` must be a named variable already in scope. This rule applies uniformly to:

| Context | Example |
|---|---|
| Function call | `add(value1, value2)` |
| Struct construction | `Room room = (area, name, occupied)` |
| Struct return | `return (quotient, remainder)` |

Order does not matter for struct construction or struct return — fields are matched by name. Order does matter for function calls, since those are positional.

There are no anonymous tuple types in Deor. `return (quotient, remainder)` constructs the function's declared return struct — both `quotient` and `remainder` must be variables in scope that match field names on that struct. See [Functions — Multiple return values](docs/functions.md#multiple-return-values) for the full pattern.

**Correct:**
```deor
struct Room
    Squarefeet area
    string name

Squarefeet area = 20
name as "Office"
Room room = (area, name)      # correct
Room room = (name, area)      # also correct — order doesn't matter for structs
```

**Incorrect — transpiler error:**
```deor
Room room = ("Office", area)  # literal not allowed — name must be a variable
```

For struct destructuring with `in`, field names drive the binding — order does not matter and any subset is valid.

---
## Named Arguments — User-Defined Functions Only
When a user-defined function is called with **2 or 3 arguments**, every argument must be a named variable already in scope. Literals, arithmetic expressions, inline function call results, and inline struct constructions are not valid in that position.

When called with **exactly 1 argument**, the argument does not need to be a named variable — a literal or expression is allowed.

**Correct:**
```deor
fn int add(int left, int right)
    return left + right

num as 5
amt as 3
int result = add(num, amt)    # 2 args — both must be named
```

```deor
fn int double(int val)
    return val * 2

int result = double(5)        # 1 arg — literal allowed
```

**Incorrect — transpiler errors:**
```deor
int result = add(5, 3)               # 2 args — literals not allowed
int result = add(num + 1, amt)       # 2 args — expression not allowed
```

`move var` counts as a named argument — passing ownership of a variable still satisfies the rule:

```deor
do_something(move big_list, other)    # valid — move var is a named argument
```

**Built-in functions** accept a single literal or expression directly. For built-ins called with 2 or more arguments, the named-variable rule still applies:

```deor
print("Hello, world!")    # 1 arg — literal fine
int cnt = len(rooms)      # 1 arg — variable fine

start as 0
stop as 10
for idx in range(start, stop)    # 2 args — named variables required
    ...
```

---
## No Nested Functions
Functions may only be declared at the top level of a file. Defining a `fn` inside another `fn` body is a transpiler error.

**Correct:**
```deor
fn bool is_valid(int val)
    return val > 0

fn string describe(int val)
    if is_valid(val)
        return "positive"
    return "invalid"
```

**Incorrect — transpiler errors:**
```deor
fn string describe(int val)
    fn bool is_valid(int num)    # not allowed
        return num > 0
    if is_valid(val)
        return "positive"
    return "invalid"
```

Move all helper functions to the top level of the file and call them by name.

---
## `raw` — Opaque Values Deor Doesn't Type-Check

A `raw` variable holds a value whose real type Deor doesn't know or track — only Rust does. `raw name = expr` must be assigned from a call to a function — a bare literal or an inline `rust` block on the right of `=` is rejected, and so is `raw name as expr` (it must be `=`, not `as`). Once declared, a raw variable can be passed to functions (as an argument, or captured from what they return) and used freely inside `rust` blocks. It can never be:

- given a different type via a typed binding or an `as` rebind
- reassigned
- used as an operand of a Deor operator (`+`, `is`, `and`, ...)
- passed to `len`, `crash`, or the bracket-literal form of `s_join` — these assume a concrete type
- declared as a struct field

Each of the above is a transpiler error. Passing a raw variable into an ordinary function call is fine — Rust's own compiler is what checks that value is used correctly there.

**Correct:**
```deor
fn LookupTable build_lookup_table()
    rust
        ...

fn void run()
    raw index = build_lookup_table()
    string result = lookup(index, search_key)    # passing raw to a function — ok
```

**Incorrect — transpiler errors:**
```deor
string val = index              # raw cannot be given another type
copy as index                   # as-rebinding is still a type capture
index = build_lookup_table()    # raw cannot be reassigned
int cnt = len(index)            # len assumes a concrete type
int total = index + 1           # operators assume a concrete type

raw bad1 = 5                    # must be a function call, not a literal
raw bad2 = rust
    42                          # must be a function call, not an inline rust block
raw bad3 as build_lookup_table() # must be '=', not 'as'

struct Config
    raw lookup_table            # raw cannot be a struct field
```
