# Welcome to Presentrs

A terminal slideshow rendered from plain **markdown** with *syntax highlighting* and a real TUI.

## What you'll see

- **Bold**, *italic*, and `inline code`
- Bulleted and numbered lists
- Tables with bold header rows
- Rust, Python, and **Lean 4** code blocks
- Horizontal rules as dividers

---

# Rust Example

```rust
fn main() {
    let greeting = "Hello, World!";
    println!("{}", greeting);

    let numbers = vec![1, 2, 3, 4, 5];
    let sum: i32 = numbers.iter().sum();

    match sum {
        15 => println!("Perfect sum!"),
        _ => println!("Sum is {}", sum),
    }
}
```

# Python Example

```python
def fibonacci(n: int) -> int:
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)

print([fibonacci(i) for i in range(10)])
```

# Lean 4 Example

Presentrs has a dedicated Lean 4 highlighter.

```lean4
import Mathlib.Data.Nat.Basic

namespace Demo

/-- Addition on `Nat` is commutative. -/
theorem add_comm (a b : Nat) : a + b = b + a := by
  induction a with
  | zero      => simp
  | succ n ih => simp [Nat.succ_add, ih]

@[simp] def double (n : Nat) : Nat := n + n

example : ∀ n : Nat, double n = 2 * n := by
  intro n
  simp [double, Nat.two_mul]

end Demo
```

Notice the coloring on `theorem`, `by`, `simp`, `Nat`, `∀`, and the `/-- … -/` doc comment.

# Lists — Ordered and Unordered

## Checklist

- Bold **items** for emphasis
- *Italic* items for style
- `Code` items for technical content

## Steps

1. Write a markdown file
2. Add `#` headings for each slide
3. Run `cargo run -- your-talk.md`
4. Navigate with `→` / `←` (or `h` / `l`)

# Tables

| Feature | Status | Notes |
|---------|--------|-------|
| Bold text | Working | **Looks great** |
| Italic text | Working | *Very nice* |
| Code blocks | Working | `Highlighted` |
| Lean 4 | Working | Custom highlighter |
| Tables | Working | Bold headers, Unicode borders |

# Navigation Reference

| Key | Action |
|-----|--------|
| `→`, `l`, `Space`, `PageDown` | Next slide |
| `←`, `h`, `PageUp` | Previous slide |
| `↓`, `j` | Scroll down |
| `↑`, `k` | Scroll up |
| `Home` / `End` | First / last slide |
| `q`, `Esc` | Quit |

# Thank You

Presentrs renders:

- **Bold** and *italic* text
- `Inline code` with a subtle background
- Headings in hierarchical colors
- Syntax-highlighted code (including **Lean 4**)
- Tables with bold headers
- Horizontal rules

---

*Press `q` to exit.*
