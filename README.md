# Presentrs

A terminal markdown slideshow renderer built with [Ratatui](https://ratatui.rs/), [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark), and [syntect](https://github.com/trishume/syntect).

**Key Rule**: Every H1 heading (`# Title`) starts a new slide.

## Quick Start

```bash
cargo run --release -- README.md
```

Any `.md` file works:

```bash
cargo run --release -- my-talk.md
```

## Navigation

| Key | Action |
|-----|--------|
| `→`, `l`, `Space`, `PageDown` | Next slide |
| `←`, `h`, `PageUp` | Previous slide |
| `↓`, `j` | Scroll down within slide |
| `↑`, `k` | Scroll up within slide |
| `Home` | Jump to first slide |
| `End` | Jump to last slide |
| `q`, `Esc` | Quit |

The slide counter and full keybinding hint are shown in the status bar at the bottom. Resizing the terminal while presenting is supported — slides re-render automatically so H1 titles stay centered.

## Supported Markdown

- **Headings**: H1 creates slides; H2–H6 style content within the current slide.
- **Paragraphs**, **soft/hard breaks**, and **horizontal rules** (`---`).
- **Lists**: bulleted and numbered, with nested-indentation preserved.
- **Emphasis**: `*italic*`, `**bold**`, `` `inline code` ``.
- **Code blocks** with syntax highlighting for 50+ languages (see below).
- **Tables** with Unicode box borders and **bold header rows**.

## Syntax Highlighting

Code blocks are highlighted by `syntect`'s default syntax set, covering languages including:

- **Systems**: `rust`, `c`, `cpp`/`c++`/`cxx`, `go`, `asm`
- **Web**: `javascript`/`js`, `typescript`/`ts`, `html`, `css`, `php`
- **Enterprise**: `java`, `csharp`/`c#`/`cs`, `scala`, `kotlin`/`kt`, `groovy`
- **Scripting**: `python`/`py`, `ruby`/`rb`, `perl`/`pl`, `lua`, `powershell`/`ps1`, `shell`/`bash`/`sh`
- **Functional**: `haskell`/`hs`, `elixir`/`ex`, `erlang`/`erl`, `clojure`/`clj`, `ocaml`, `fsharp`/`f#`, `lisp`, `scheme`
- **Proof assistants**: `lean`/`lean4` (custom highlighter — see below)
- **Mobile**: `swift`, `dart`
- **Data/Config**: `json`, `xml`, `yaml`/`yml`, `toml`, `ini`, `properties`
- **Docs**: `markdown`/`md`, `tex`/`latex`
- **Other**: `sql`, `dockerfile`, `makefile`, `nginx`, `apache`, `vim`, `r`, `matlab`, `verilog`, `vhdl`

### Lean 4

Presentrs ships a dedicated Lean 4 highlighter — `syntect` does not include a Lean 4 grammar. Fence a block with `lean` or `lean4` to activate it:

````markdown
```lean4
theorem add_comm (a b : Nat) : a + b = b + a := by
  induction a with
  | zero => simp
  | succ n ih => simp [Nat.succ_add, ih]
```
````

The highlighter recognises:

- Keywords (`def`, `theorem`, `structure`, `namespace`, `by`, `match`, …)
- Tactics (`simp`, `rw`, `exact`, `apply`, `induction`, `rcases`, `omega`, …)
- Built-in types / sorts (`Prop`, `Type`, `Sort`, `Nat`, `List`, `Option`, …)
- Line comments (`-- …`), nested block comments (`/- /- … -/ -/`)
- String and character literals, numeric literals
- Attribute forms like `@[simp]`
- Lean 4 unicode operators: `∀`, `∃`, `λ`, `→`, `↔`, `∧`, `∨`, `¬`, `≤`, `⟨⟩`, `⊢`, and more

## Example

Create `slides.md`:

```markdown
# Welcome
Intro content

# Problem
- Bullet
- Another bullet

# Solution

```rust
fn main() {
    println!("Hello");
}
```

# Thanks
```

Run it:

```bash
cargo run --release -- slides.md
```

## Further Reading

- [`TUTORIAL.md`](./TUTORIAL.md) — a walkthrough of every feature, delivered as its own slideshow.
- [`demo.md`](./demo.md) — a small end-to-end demo including Lean 4.
