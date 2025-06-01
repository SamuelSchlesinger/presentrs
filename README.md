# Understanding the Slideshow System

This markdown file demonstrates how the terminal slideshow system works.

**Key Concept**: Every H1 heading (`# Title`) creates a new slide!

## Quick Start

```bash
cargo run README.md  # This file becomes a slideshow!
```

## Navigation

| Key | Action |
|-----|--------|
| `→`, `l`, `Space` | Next slide |
| `←`, `h` | Previous slide |
| `q`, `Esc` | Quit |

---

# Slide Creation Rule

**This is Slide 2** because it starts with `# Slide Creation Rule`

Everything between H1 headings becomes part of the same slide:

- This bullet point is on slide 2
- So is this one
- And any other content until the next H1

## Subsections Work Too

H2, H3, H4, H5, H6 headings don't create new slides - they organize content within the current slide.

Press `→` to see the next slide!

# Content Types Supported

**This is Slide 3** - notice how we jumped here from slide 2?

## Text Formatting

You can use **bold text** and *italic text* in your slides.

`Inline code` works great for technical content.

## Lists Are Perfect

• Bullet points for key information
• Multiple levels of organization  
• Clear, readable formatting

## Code Blocks

```rust
fn main() {
    println!("Code blocks maintain formatting!");
}
```

# How H1 Headings Work

**This is Slide 4** - created by the H1 `# How H1 Headings Work`

## The Magic Behind Slides

The slideshow parser looks for lines that start with `# ` (H1 markdown syntax).

When it finds one, it:
1. Ends the current slide (if any)
2. Starts a new slide with that H1 as the title
3. Includes all content until the next H1

## Example Structure

```markdown
# First Slide
Content for slide 1

# Second Slide  
Content for slide 2

# Third Slide
Content for slide 3
```

This creates exactly 3 slides!

# Building Your Own Slideshow

**This is Slide 5** - let's learn how to create presentations

## Step 1: Write Markdown

Create any `.md` file with H1 headings:

```markdown
# My Presentation Title
Introduction content here

# Problem Statement  
Describe the problem

# Solution
Your brilliant solution

# Thank You
Closing remarks
```

## Step 2: Run the Slideshow

```bash
cargo run my-presentation.md
```

## Step 3: Present!

Navigate with arrow keys, space, or `h`/`l` (vim-style)

# Advanced Slide Techniques

**This is Slide 6** - pro tips for better presentations

## Organize Content with Subheadings

### This is an H3
#### This is an H4  
##### This is an H5

All these subheadings stay on the same slide because they're not H1!

## Mix Different Content Types

| Feature | Status |
|---------|--------|
| Tables | ✅ Supported |
| Lists | ✅ Supported |
| Code | ✅ Supported |

1. Numbered lists work too
2. Great for step-by-step processes
3. Mix with other content freely

# Implementation Details

**This is Slide 7** - how the system actually works

## The Parser Logic

The slideshow system scans your markdown file looking for:

```rust
// Simplified logic
if line.starts_with("# ") {
    // Start new slide
    slides.push(new_slide);
} else {
    // Add to current slide
    current_slide.add_content(line);
}
```

## Supported Markdown Elements

| Element | Support | Notes |
|---------|---------|-------|
| Headings | ✅ H1-H6 | **Only H1 creates slides** |
| Paragraphs | ✅ | Standard text blocks |
| Lists | ✅ | Bulleted and numbered |
| Emphasis | ✅ | *italic* and **bold** |
| Code | ✅ | `inline` and ```blocks``` |
| Tables | ✅ | Like this one! |

## Built With Rust

- **Ratatui**: Terminal UI framework
- **pulldown-cmark**: Markdown parsing
- **Crossterm**: Cross-platform terminal control

# Try It Yourself!

**This is the Final Slide** - time to experiment

## Create Your First Slideshow

1. Make a new `.md` file
2. Add some H1 headings
3. Fill with content
4. Run `cargo run your-file.md`

## Example Template

```markdown
# Welcome
Your introduction here

# Main Topic
Core content

# Conclusion  
Wrap up thoughts
```

## Remember the Golden Rule

**Every H1 (`# Title`) = New Slide**

Everything else organizes content within slides.

**Now go create amazing terminal presentations!**

---

*This presentation demonstrates itself - press `q` to exit*