# Presentrs Tutorial: Terminal Markdown Slideshows

Welcome to Presentrs! This comprehensive tutorial will walk you through every feature of this terminal-based markdown slideshow tool.

**Key Rule**: Every H1 heading (`# Title`) creates a new slide!

## Getting Started

Run this tutorial as a slideshow:

```bash
cargo run TUTORIAL.md
```

### Navigation Controls

| Key | Action |
|-----|--------|
| `→`, `l`, `Space` | Next slide |
| `←`, `h` | Previous slide |
| `↑`, `k` | Scroll up within slide |
| `↓`, `j` | Scroll down within slide |
| `q`, `Esc` | Quit |

# Slide Creation Basics

**This is Slide 2** - created by the H1 heading above

## The Golden Rule

Every H1 heading (`# Title`) starts a new slide. Everything between H1 headings becomes part of the same slide.

## What Goes on a Slide

- All H2-H6 headings stay on the current slide
- Paragraphs, lists, code blocks, tables
- Any content until the next H1 heading

### Example Structure

```markdown
# First Slide
Content for slide 1

# Second Slide  
Content for slide 2
```

This creates exactly 2 slides!

# Text Formatting Features

**This is Slide 3** - demonstrating text styling

## Basic Formatting

You can use **bold text** for emphasis and *italic text* for style.

Combine them: ***bold and italic*** text together.

## Inline Code

Use `inline code` for technical terms, variable names, or short code snippets. It appears with a distinctive background color.

## Paragraphs

Regular paragraphs flow naturally and wrap to fit your terminal width. 

Multiple paragraphs are separated by blank lines for clear visual distinction.

# Heading Hierarchy

**This is Slide 4** - showing all heading levels

## H2 Heading (Blue, Bold)
### H3 Heading (Green, Bold)  
#### H4 Heading (Yellow, Bold)
##### H5 Heading (Yellow, Bold)
###### H6 Heading (Yellow, Bold)

Notice how:
- H1 headings are **cyan and centered** (slide titles)
- H2 headings are **blue and bold**
- H3 headings are **green and bold** 
- H4-H6 headings are **yellow and bold**

All non-H1 headings organize content within the current slide.

# Lists and Organization

**This is Slide 5** - demonstrating list features

## Bulleted Lists

Presentrs renders markdown lists with attractive bullet points:

• First item with bullet point
• Second item 
• Third item with longer text that wraps naturally in your terminal

## Nested Organization

You can combine lists with other elements:

• **Bold items** for emphasis
• *Italic items* for style
• `Code items` for technical content
• Regular items for normal text

Lists automatically get proper spacing and the distinctive yellow bullet character (•).

# Code Blocks and Syntax Highlighting  

**This is Slide 6** - showcasing code features

## Basic Code Blocks

```
Plain code blocks without language specification
Still get monospace formatting
Great for simple examples
```

## Rust Code Example

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

Notice the beautiful syntax highlighting with proper colors!

# Advanced Rust Examples

**This is Slide 7** - more complex code

## Structs and Implementations

```rust
use std::collections::HashMap;

struct Person {
    name: String,
    age: u32,
}

impl Person {
    fn new(name: &str, age: u32) -> Self {
        Person {
            name: name.to_string(),
            age,
        }
    }
    
    fn greet(&self) -> String {
        format!("Hi, I'm {} and I'm {} years old", self.name, self.age)
    }
}

fn main() {
    let mut people = HashMap::new();
    people.insert("alice", Person::new("Alice", 30));
    people.insert("bob", Person::new("Bob", 25));
    
    for (key, person) in &people {
        println!("{}: {}", key, person.greet());
    }
}
```

# Multi-Language Code Support

**This is Slide 8** - showing language variety

## Python Example

```python
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

numbers = [fibonacci(i) for i in range(10)]
print(f"First 10 Fibonacci numbers: {numbers}")
```

## JavaScript Example

```javascript
const fetchData = async (url) => {
    try {
        const response = await fetch(url);
        const data = await response.json();
        return data;
    } catch (error) {
        console.error('Error fetching data:', error);
    }
};
```

## Go Example

```go
package main

import (
    "fmt"
    "time"
)

func main() {
    ch := make(chan string)
    
    go func() {
        time.Sleep(2 * time.Second)
        ch <- "Hello from goroutine!"
    }()
    
    message := <-ch
    fmt.Println(message)
}
```

# Table Features

**This is Slide 9** - demonstrating table rendering

## Language Support Table

| Language | Extension | Status | Notes |
|----------|-----------|--------|-------|
| **Systems** |  |  |  |
| Rust | `.rs` | ✅ | Primary language |
| C | `.c` | ✅ | Classic systems |
| C++ | `.cpp` | ✅ | Object-oriented |
| Go | `.go` | ✅ | Modern concurrent |
| **Web** |  |  |  |
| JavaScript | `.js` | ✅ | Frontend/backend |
| TypeScript | `.ts` | ✅ | Typed JavaScript |
| HTML | `.html` | ✅ | Markup |
| CSS | `.css` | ✅ | Styling |

## Feature Comparison

| Feature | Supported | Quality | Notes |
|---------|-----------|---------|-------|
| Text formatting | ✅ | Excellent | **Bold**, *italic*, `code` |
| Syntax highlighting | ✅ | Excellent | 40+ languages |
| Tables | ✅ | Good | Unicode borders |
| Navigation | ✅ | Excellent | Vim-style keys |
| Scrolling | ✅ | Good | Long slide support |

# Comprehensive Language Support

**This is Slide 10** - complete language reference

## Systems Programming
- **rust, c, cpp/c++/cxx, go, assembly/asm**

## Web Development  
- **javascript/js, typescript/ts, html, css, php**

## Enterprise Languages
- **java, csharp/c#/cs, scala, kotlin/kt, groovy**

## Scripting Languages
- **python/py, ruby/rb, perl/pl, lua, powershell/ps1, shell/bash/sh**

## Functional Languages
- **haskell/hs, elixir/ex, erlang/erl, clojure/clj, ocaml, fsharp/f#, lisp, scheme**

## Mobile Development
- **swift, dart**

## Data & Configuration
- **json, xml, yaml/yml, toml, ini, properties**

## Documentation & Markup
- **markdown/md, tex/latex**

## Specialized
- **sql, dockerfile/docker, makefile/make, nginx, apache, vim**
- **r, matlab, cobol, pascal, fortran, ada**
- **verilog, vhdl** (hardware description)

# Advanced Navigation Features

**This is Slide 11** - mastering slideshow control

## Keyboard Shortcuts Reference

### Navigation
- `→` `l` `Space` - Next slide (three ways!)
- `←` `h` - Previous slide  
- `q` `Esc` - Quit application

### Scrolling (for long slides)
- `↑` `k` - Scroll up within current slide
- `↓` `j` - Scroll down within current slide

## Pro Tips

### Vim-Style Navigation
If you're familiar with Vim, use `h`/`l` for horizontal navigation and `j`/`k` for vertical scrolling.

### Multiple Options
The tool provides multiple key options for each action, so use whatever feels most natural to you.

### Slide Counter
Notice the slide counter at the bottom: `Slide X/Y` shows your progress through the presentation.

# Creating Your Own Slideshows

**This is Slide 12** - practical slideshow creation

## Basic Template

```markdown
# Welcome Slide
Your introduction and overview

# Problem Statement
- What challenge are you addressing?
- Why does it matter?
- Who is affected?

# Proposed Solution
## Approach
Your methodology

## Implementation
Technical details with code:

```rust
fn solve_problem() {
    println!("Your solution here");
}
```

# Results
| Metric | Before | After |
|--------|--------|-------|
| Performance | Slow | Fast |
| Usability | Poor | Great |

# Conclusion
Summary and next steps
```

## Best Practices

### Slide Structure
- **One main idea per slide**
- Use H2/H3 for organization within slides
- Keep slides focused and concise

### Visual Elements
- **Bold** for emphasis
- *Italic* for subtlety  
- `Code` for technical terms
- Tables for comparisons
- Lists for organization

# Advanced Presentation Techniques

**This is Slide 13** - professional presentation tips

## Content Organization

### Opening Strong
- Start with a compelling H1 title
- Include overview/agenda slide
- Set clear expectations

### Building Narrative
- Logical flow between slides
- Use consistent formatting
- Progressive disclosure of information

### Closing Effectively  
- Summarize key points
- Include next steps or call to action
- Thank your audience

## Technical Presentations

### Code Demonstrations
```rust
// Use meaningful examples
fn calculate_fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => calculate_fibonacci(n - 1) + calculate_fibonacci(n - 2),
    }
}

// Add explanatory comments
let result = calculate_fibonacci(10); // Calculate 10th Fibonacci number
println!("The 10th Fibonacci number is: {}", result);
```

### Data Presentation

| Quarter | Revenue | Growth |
|---------|---------|--------|
| Q1 2024 | $100K | +15% |
| Q2 2024 | $120K | +20% |
| Q3 2024 | $150K | +25% |

# Interactive Features and Tips

**This is Slide 14** - maximizing your presentation experience

## Live Demonstration Features

### Real-Time Scrolling
This slide intentionally contains more content than fits on one screen to demonstrate the scrolling feature.

When slide content exceeds your terminal height, use:
- `↓` or `j` to scroll down
- `↑` or `k` to scroll up

This allows you to present detailed information while maintaining readability.

### Terminal Responsiveness
- Resize your terminal to see content adapt
- Content flows naturally to fit width
- Tables and code blocks maintain formatting

## Presentation Environment Setup

### Terminal Configuration
- Use a dark terminal theme for better contrast
- Increase font size for better visibility
- Maximize terminal window for content

### Screen Sharing
- Terminal presentations work great for screen sharing
- Content remains crisp at any zoom level
- No dependency on external presentation software

# Customization and Extensions

**This is Slide 15** - advanced usage scenarios

## File Organization

### Project Structure
```
presentation/
├── slides.md          # Main presentation
├── appendix.md        # Detailed technical info  
├── demo.md           # Live demo script
└── notes.md          # Speaker notes
```

### Modular Presentations
Break complex presentations into focused files:
- `intro.md` - Opening and overview
- `technical.md` - Deep technical content
- `results.md` - Data and outcomes
- `conclusion.md` - Summary and next steps

## Content Strategies

### Progressive Complexity
```markdown
# Basic Concept
Simple introduction

# Intermediate Details  
More depth with examples

# Advanced Implementation
Full technical implementation
```

### Mixed Content Types
Combine different markdown elements effectively:
- **Headers** for structure
- **Lists** for organization
- **Tables** for data
- **Code** for examples
- **Text** for explanation

# Technical Implementation Details

**This is Slide 16** - understanding how Presentrs works

## Core Architecture

### Built With Rust
- **ratatui** - Terminal UI framework
- **pulldown-cmark** - Markdown parsing
- **crossterm** - Cross-platform terminal control
- **syntect** - Syntax highlighting engine
- **clap** - Command-line interface

### Key Components

```rust
struct App {
    slides: Vec<Text<'static>>,    // Parsed slide content
    current_slide: usize,          // Active slide index  
    scroll_offset: usize,          // Vertical scroll position
    theme_set: ThemeSet,           // Syntax highlighting themes
    syntax_set: SyntaxSet,         // Language definitions
}
```

## Parsing Process

### Slide Detection
1. Scan markdown for H1 headings (`# Title`)
2. Split content at each H1 boundary
3. Parse markdown within each slide section
4. Apply syntax highlighting to code blocks
5. Render tables with Unicode borders

### Performance Features
- Lazy syntax highlighting initialization
- Efficient Unicode width calculations
- Minimal redraws on user interaction

# Real-World Use Cases

**This is Slide 17** - practical applications

## Technical Presentations

### Code Reviews
- Present code changes slide by slide
- Include before/after comparisons
- Add explanatory context

### Architecture Discussions
```markdown
# Current Architecture
Problems and limitations

# Proposed Changes
Technical improvements

# Implementation Plan
Step-by-step approach
```

### Training Sessions
- Interactive code walkthroughs
- Live coding demonstrations
- Progressive skill building

## Business Presentations

### Project Updates
| Sprint | Feature | Status |
|--------|---------|--------|
| Sprint 1 | User Auth | ✅ Complete |
| Sprint 2 | Dashboard | 🟡 In Progress |
| Sprint 3 | Reports | ⏳ Planned |

### Technical Demos
- Live terminal demonstrations
- Real command execution
- Authentic development environment

# Best Practices Summary

**This is Slide 18** - key takeaways for effective slideshows

## Content Guidelines

### Structure
- **One H1 = One Slide** (the golden rule)
- Use H2-H6 for organization within slides
- Keep slides focused on single concepts

### Formatting
- **Bold** for emphasis and key terms
- *Italic* for subtle emphasis or definitions
- `Code` for technical terms and variables
- Lists for organization and clarity

### Code Presentation
- Choose appropriate language syntax highlighting
- Include meaningful comments in code examples
- Keep code blocks concise and focused
- Use descriptive variable names

## Presentation Tips

### Preparation
- Test your slideshow before presenting
- Practice navigation timing
- Prepare for potential questions

### Delivery
- Use consistent navigation rhythm
- Pause at complex slides for questions
- Leverage scrolling for detailed content

### Audience Engagement
- Include interactive elements
- Ask questions between slides
- Encourage terminal/command line exploration

# Advanced Markdown Features

**This is Slide 19** - exploring edge cases and special formatting

## Complex Table Layouts

### Multi-Row Headers

| Category | Subcategory | Feature | Support |
|----------|-------------|---------|---------|
| **Text** | Basic | Bold/Italic | ✅ |
|          | Advanced | `Inline code` | ✅ |
| **Code** | Blocks | Syntax highlighting | ✅ |
|          | Languages | 40+ supported | ✅ |
| **Layout** | Tables | Unicode borders | ✅ |
|           | Lists | Bullet formatting | ✅ |

### Data Tables

| Language | Lines of Code | Compile Time | Memory Usage |
|----------|---------------|--------------|--------------|
| Rust | 770 | 2.3s | 15MB |
| Go | 450 | 0.8s | 25MB |
| Python | 320 | N/A | 45MB |
| C++ | 890 | 4.1s | 12MB |

## Mixed Content Examples

### Documentation with Code

The `parse_markdown_to_slides` function handles complex parsing:

```rust
fn parse_markdown_to_slides(
    markdown: &str,
    theme_set: &ThemeSet,
    syntax_set: &SyntaxSet,
    terminal_width: u16,
) -> Vec<Text<'static>> {
    // Implementation details...
}
```

Key features:
- **State machine parsing** for robust handling
- **Syntax highlighting integration** with syntect
- **Unicode-aware text width** calculations
- **Memory-efficient slide storage**

# Troubleshooting and Common Issues

**This is Slide 20** - solving presentation problems

## Common Problems

### Slide Not Creating
**Problem**: Content not appearing on separate slides
**Solution**: Ensure you're using H1 (`# Title`) not H2 (`## Title`)

```markdown
# This creates a slide ✅
## This stays on current slide ❌
```

### Code Not Highlighting  
**Problem**: Code blocks appear without colors
**Solution**: Specify language after triple backticks

```markdown
```rust  ✅
```javascript  ✅  
```  ❌ (no highlighting)
```

### Table Formatting Issues
**Problem**: Tables appear malformed
**Solution**: Ensure proper markdown table syntax

```markdown
| Header 1 | Header 2 |  ✅
|----------|----------|
| Cell 1   | Cell 2   |

Header 1 | Header 2    ❌ (missing separators)
Cell 1   | Cell 2
```

## Performance Tips

### Large Presentations
- Break into multiple files for easier management
- Use descriptive slide titles for navigation
- Keep individual slides concise

### Terminal Optimization
- Use monospace fonts for best results
- Adjust terminal size for comfortable viewing
- Consider dark themes for better contrast

# Future Enhancements and Extensibility

**This is Slide 21** - potential improvements and customization

## Possible Enhancements

### Configuration Options
- Custom color themes
- Configurable key bindings  
- Font size adjustments
- Slide transition effects

### Extended Features
- Image support (ASCII art)
- Mathematical formula rendering
- Interactive elements
- Presenter notes display

### Export Capabilities
- PDF generation
- HTML export
- Static site generation
- Video recording support

## Customization Ideas

### Theme Development
```rust
// Custom color scheme
let custom_theme = Theme {
    h1_color: Color::Magenta,
    h2_color: Color::Cyan,
    code_bg: Color::Rgb(20, 20, 20),
    // Additional customizations...
};
```

### Plugin Architecture
- Custom markdown extensions
- Third-party syntax highlighting
- Integration with external tools
- Collaborative editing features

# Thank You!

**Final Slide** - congratulations on completing the tutorial

## You've Learned

✅ **Slide Creation** - Using H1 headings to structure presentations  
✅ **Text Formatting** - Bold, italic, and inline code styling  
✅ **Code Blocks** - Syntax highlighting for 40+ languages  
✅ **Tables** - Professional data presentation with Unicode borders  
✅ **Navigation** - Efficient keyboard controls and scrolling  
✅ **Best Practices** - Professional presentation techniques  

## Next Steps

### Start Creating
1. Create a new `.md` file
2. Add H1 headings for slides
3. Fill with your content
4. Run `cargo run your-presentation.md`

### Practice Examples
- Technical documentation walkthrough
- Project status presentation  
- Code review discussion
- Training material development

### Share Your Experience
The terminal-based approach offers unique advantages:
- **Cross-platform compatibility**
- **Version control friendly**
- **Lightweight and fast**
- **Developer-focused workflow**

## Remember the Golden Rule

**Every H1 (`# Title`) = New Slide**

Everything else organizes content within slides.

**Now go create amazing terminal presentations!**

---

*Press `q` to exit this tutorial slideshow*