<!-- title: Deor Specification -->
<!-- [Deor Specification Index](index.md) -->
<!-- themes: dusk -->
# Macros

A macro is a named block of code that is inlined at every `macro_run` call site — equivalent to copy-pasting the macro body at that point in the source.

Macros are useful because they avoid the clone overhead of function calls while still letting you organize repetitive logic. This matters most inside tight loops where calling helper functions repeatedly introduces unnecessary cloning. Unlike Rust macros, Deor macros need no parameter declarations and require no special syntax — they benefit from Rust's compile-time checks because the inlined code goes through the same transpile and compile pipeline.

Because macro bodies are inlined, any variables they declare pollute the caller's scope. Use the `block` keyword inside a macro body to contain variables that should not escape.

Macros can be declared at the top level or inside a function body. A top-level macro is available to any file that imports the file it is defined in — no special import syntax needed. A macro declared inside a function body is locally scoped: it exists only within that block and is not visible outside it.

You can call other macros with `macro_run` from inside a macro body — nesting calls is fully supported. However, defining a `macro` inside another macro body is a compile error. Define all macros at the appropriate scope and call them with `macro_run`.

```deor
macro say_hello
    print(hello)

fn void greet()
    hello as "Hi There"
    macro_run say_hello

    hello as "Hi There Again"
    macro_run say_hello

# output is "Hi There"
# and "Hi There Again"
```

---

## `block` Inside Macros

Because a macro body is copy-pasted at the call site, any variables it declares become part of the caller's scope. If the macro is called more than once, or if its internal variable names conflict with the caller's names, this causes a compile error.

Use `block` inside the macro body to create an isolated scope. Variables declared inside `block` do not escape:

Deor:
```deor
macro compute_area
    block
        length as 10
        width as 5
        area as length * width
        print(area)

fn void run()
    macro_run compute_area
    macro_run compute_area    # safe — block variables do not leak between calls
```

Rust:
```rust
{
    let length = 10;
    let width = 5;
    let area = length * width;
    println!("{}", area);
}
{
    let length = 10;
    let width = 5;
    let area = length * width;
    println!("{}", area);
}
```

Without `block`, the second `macro_run` would fail to compile because `length`, `width`, and `area` would already be declared in scope.

If the macro only reads variables from the caller's scope without declaring any of its own, `block` is not needed.

---

## Convention: Wrapping a Body With a Start/End Pair

There's no dedicated syntax for wrapping arbitrary code between "before" and "after" snippets — just define two macros and call them in sequence, like this:

```deor
macro timer_start
    int _timer_start = now_ms()

macro timer_end
    int _timer_elapsed = elapsed_ms(_timer_start)
    string _timer_str = n_to_str(_timer_elapsed)
    string _timer_sfx = "ms"
    print(s_join([_timer_label, _timer_str, _timer_sfx]))
```

```deor
string _timer_label = "[timer] load: "
macro_run timer_start
tokenList raw_tokens = collect_all_tokens_with_all_imports(input_path)
macro_run timer_end
```

`timer_end` freely reads `_timer_start` (declared by `timer_start`) and `_timer_label` (set by the caller) since both live in the same inlined scope. Nothing enforces that `timer_end` actually gets called — that's just the tradeoff of macros inlining into caller scope, same as any other macro.
