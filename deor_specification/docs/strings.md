<!-- title: Deor Specification -->
<!-- [Deor Specification Index](index.md) -->
<!-- themes: dusk -->
# Strings

String utility functions live in `lib/string.deor`. Import the whole file:

```deor
import "lib/string.deor"
```

All functions in `lib/string.deor` follow the `s_` prefix convention. They are regular user-defined functions, so arguments must be named variables — literals are not valid arguments. See [Enforced Practices](docs/enforced_practices.md#named-arguments-user-defined-functions-only).

---

## Escape Sequences

Standard escape sequences are supported inside string literals:

| Sequence | Meaning |
|---|---|
| `\n` | Newline |
| `\t` | Horizontal tab |
| `\\` | Literal backslash |
| `\"` | Literal double quote |

```deor
msg as "Hello\nWorld"
path as "C:\\Users\\name"
quote as "She said \"hello\""
```

```rust
let msg = "Hello\nWorld".to_string();
let path = "C:\\Users\\name".to_string();
let quote = "She said \"hello\"".to_string();
```

No other escape sequences are supported in v1. For Unicode escapes or raw byte strings, use a `rust` block.

---

## Concatenation

`+` joins strings — but it's a convenience for chains that already have a literal in them, not a general-purpose join. It works with literals, variables, or any combination, as long as the chain contains at least one string literal somewhere:

```deor
string greeting = "hello " + name
string line = prefix + content + "\n"
string full = first + " " + last
```

```rust
let greeting: String = ["hello ", name.as_str()].concat();
let line: String = [prefix.as_str(), content.as_str(), "\n"].concat();
let full: String = [first.as_str(), " ", last.as_str()].concat();
```

Chains of `+` are evaluated left to right and compiled to `[...].concat()` — the same shape `s_join`'s bracket-literal form produces below — not a native Rust `+`/`&` chain and not `format!`. Every operand is borrowed (`.as_str()`), except string literals, which are already `&str`. Nothing is ever cloned or moved, so every variable in the chain — first or not — stays usable afterward, and `move` has no effect on any of them (see [Move — Move in String Concatenation](docs/move.md#move-in-string-concatenation)).

**This only works because a literal is present.** Two plain string variables with no literal anywhere — `full = aaa + bbb` — are not recognized as a concat chain and fail to compile as plain arithmetic. This is a known, accepted gap: `+` is sugar for the literal-mixed case. For joining variables with no literal involved, use `s_join`/`s_join_with` instead (see the table below) — it always works and is the recommended way to join a list of strings in general:

```deor
string full = s_join([aaa, bbb])
```

The transpiler does check that operands *directly adjacent* to a `+` aren't mixing a string with a number (`"count: " + count` is caught at the Deor level with a clear error), but that check only looks at each `+`'s immediate neighbors, so it won't catch every possible mismatch in a longer chain — that residual case still surfaces as a Rust type error. Use a `rust` block if you need to format an integer into a string:

```deor
fn string int_to_str(int n)
    rust
        n.to_string()

string msg = "count: " + int_to_str(count)
```

---

## Examples

```deor
import "lib/string.deor"

string raw = "  Hello, World!  "
string clean = s_trim(raw)

string lower = s_to_lower(clean)
string query = "world"
bool found = s_contains(lower, query)

string csv = "apple,banana,cherry"
string sep = ","
stringList parts = s_split(csv, sep)
```

```rust
let raw: String = "  Hello, World!  ".to_string();
let clean: String = raw.trim().to_string();
let lower: String = clean.to_lowercase();
let query: String = "world".to_string();
let found: bool = lower.contains(query.as_str());
let csv: String = "apple,banana,cherry".to_string();
let sep: String = ",".to_string();
let parts: Vec<String> = csv.split(sep.as_str()).map(|s| s.to_string()).collect();
```

```deor
string path = "/api/users"
string slash = "/"
bool is_abs = s_starts_with(path, slash)

string filename = "report.pdf"
string ext = ".pdf"
bool is_pdf = s_ends_with(filename, ext)
```

`s_split` always returns at least one element — an input with no delimiter occurrences returns a single-element list containing the original string.

---

## Conversion Notes

| Deor | Rust |
|---|---|
| `"lit" + b` | `["lit", b.as_str()].concat()` |
| `s_contains(str, needle)` | `str.contains(needle.as_str())` |
| `s_starts_with(str, prefix)` | `str.starts_with(prefix.as_str())` |
| `s_ends_with(str, suffix)` | `str.ends_with(suffix.as_str())` |
| `s_trim(str)` | `str.trim().to_string()` |
| `s_to_upper(str)` | `str.to_uppercase()` |
| `s_to_lower(str)` | `str.to_lowercase()` |
| `s_split(str, delimiter)` | `str.split(delimiter.as_str()).map(\|s\| s.to_string()).collect()` |
| `s_join(parts)` | `parts.join("")` |
| `s_join_with(parts, sep)` | `parts.join(sep.as_str())` |
| `s_trim_start(str)` | `str.trim_start().to_string()` |
| `s_trim_end(str)` | `str.trim_end().to_string()` |
| `s_replace(str, from, dest)` | `str.replace(from.as_str(), dest.as_str())` |
| `s_index_of(str, needle)` | `str.find(needle.as_str()).map(\|i\| i as i64).unwrap_or(-1)` |
| `s_char_at(str, idx)` | `str.chars().nth(idx as usize).map(\|c\| c.to_string()).unwrap_or_default()` |
| `s_substring(str, start, end)` | `str.chars().skip(start).take(end - start).collect()` |
| `s_repeat(str, count)` | `str.repeat(count as usize)` |
