# Welcome to Enhanced Markdown Presentation

This presentation tool now supports **proper markdown formatting** and *syntax highlighting*!

## Features

• **Bold text** for emphasis
• *Italic text* for style  
• `Inline code` highlighting
• Rust syntax highlighting in code blocks

# Rust Code Example

Here's some Rust code with syntax highlighting:

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

# Advanced Rust Features

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

# Table Example

Here's a comparison table:

| Feature | Status | Notes |
|---------|--------|-------|
| Bold text | ✅ Working | **Looks great** |
| Italic text | ✅ Working | *Very nice* |
| Code blocks | ✅ Working | `Highlighted` |
| Tables | ✅ Working | This table! |

# Thank You!

The presentation tool now renders:
• **Bold** and *italic* text properly
• `Code` with background highlighting  
• **Headers** in different colors and sizes
• Beautiful Rust syntax highlighting
• **Tables** with proper formatting