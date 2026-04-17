//! # Presentation
//!
//! A terminal-based markdown slideshow presentation tool built with Ratatui.
//!
//! This application allows you to present markdown files as slides in your terminal.
//! Slides are automatically separated by H1 headings (`# Title`), and you can navigate
//! between them using keyboard controls.
//!
//! ## Usage
//!
//! ```bash
//! cargo run <markdown-file.md>
//! ```
//!
//! ## Keyboard Controls
//!
//! - `→`, `l`, `Space`: Next slide
//! - `←`, `h`: Previous slide
//! - `↑`, `k`: Scroll up within slide
//! - `↓`, `j`: Scroll down within slide
//! - `q`, `Esc`: Quit
//!
//! ## Markdown Support
//!
//! The application supports basic markdown formatting including:
//! - Headings (H1-H6)
//! - Paragraphs
//! - Lists (bulleted)
//! - Emphasis (*italic*, **bold**)
//! - Inline code (`code`)
//! - Code blocks (```code```)

use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use pulldown_cmark::{
    Event as MarkdownEvent, HeadingLevel, Options, Parser as MarkdownParser, Tag, TagEnd,
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::{
    error::Error,
    fs,
    io::{self, Stdout},
};
use syntect::{
    easy::HighlightLines, highlighting::ThemeSet, parsing::SyntaxSet, util::LinesWithEndings,
};
use unicode_width::UnicodeWidthStr;

/// Command line arguments for the presentation tool.
#[derive(Parser)]
#[command(name = "presentation")]
#[command(about = "A terminal-based markdown slideshow presentation tool")]
struct Args {
    /// Path to the markdown file to present
    #[arg(help = "Path to the markdown file")]
    file: String,
}

/// The main application state for the slideshow.
///
/// Manages the collection of slides and tracks the current slide position.
struct App {
    /// Raw markdown content, retained so slides can be re-parsed on resize.
    markdown_content: String,
    /// Collection of slide content as formatted text
    slides: Vec<Text<'static>>,
    /// Index of the currently displayed slide (0-based)
    current_slide: usize,
    /// Vertical scroll offset for the current slide
    scroll_offset: usize,
    /// Syntax highlighting theme set
    theme_set: ThemeSet,
    /// Syntax definitions
    syntax_set: SyntaxSet,
}

impl App {
    /// Creates a new App instance from markdown content.
    fn new(markdown_content: String, terminal_width: u16) -> Self {
        let theme_set = ThemeSet::load_defaults();
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let slides =
            parse_markdown_to_slides(&markdown_content, &theme_set, &syntax_set, terminal_width);
        App {
            markdown_content,
            slides,
            current_slide: 0,
            scroll_offset: 0,
            theme_set,
            syntax_set,
        }
    }

    /// Re-parses slides for the new terminal width, preserving the current slide index.
    fn resize(&mut self, new_width: u16) {
        let slides = parse_markdown_to_slides(
            &self.markdown_content,
            &self.theme_set,
            &self.syntax_set,
            new_width,
        );
        if !slides.is_empty() {
            self.current_slide = self.current_slide.min(slides.len() - 1);
        }
        self.scroll_offset = 0;
        self.slides = slides;
    }

    /// Advances to the next slide if available.
    ///
    /// Does nothing if already on the last slide or if no slides exist.
    fn next_slide(&mut self) {
        if !self.slides.is_empty() && self.current_slide < self.slides.len() - 1 {
            self.current_slide += 1;
            self.scroll_offset = 0;
        }
    }

    /// Goes back to the previous slide if available.
    ///
    /// Does nothing if already on the first slide.
    fn prev_slide(&mut self) {
        if self.current_slide > 0 {
            self.current_slide -= 1;
            self.scroll_offset = 0;
        }
    }

    /// Jumps to the first slide.
    fn goto_first(&mut self) {
        self.current_slide = 0;
        self.scroll_offset = 0;
    }

    /// Jumps to the last slide.
    fn goto_last(&mut self) {
        if !self.slides.is_empty() {
            self.current_slide = self.slides.len() - 1;
            self.scroll_offset = 0;
        }
    }

    /// Scrolls down within the current slide.
    ///
    /// Increases the scroll offset to show content below the current view.
    fn scroll_down(&mut self) {
        if !self.slides.is_empty() {
            let max_scroll = self.slides[self.current_slide].lines.len().saturating_sub(1);
            if self.scroll_offset < max_scroll {
                self.scroll_offset += 1;
            }
        }
    }

    /// Scrolls up within the current slide.
    ///
    /// Decreases the scroll offset to show content above the current view.
    fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    /// Returns the content of the currently displayed slide.
    ///
    /// # Returns
    ///
    /// A reference to the current slide's content, or a default empty slide if no slides exist
    fn current_slide_content(&self) -> &Text<'static> {
        if self.slides.is_empty() {
            // Return a static reference to an empty text - we'll handle this in the caller
            static EMPTY_SLIDE: std::sync::LazyLock<Text<'static>> = std::sync::LazyLock::new(|| {
                Text::from("No slides found")
            });
            &EMPTY_SLIDE
        } else {
            &self.slides[self.current_slide]
        }
    }

    /// Returns a formatted string showing current slide position.
    ///
    /// # Returns
    ///
    /// A string in the format "current/total" (e.g., "3/10")
    fn slide_info(&self) -> String {
        if self.slides.is_empty() {
            "0/0".to_string()
        } else {
            format!("{}/{}", self.current_slide + 1, self.slides.len())
        }
    }
}

/// Parses markdown content into individual slides.
///
/// Slides are separated by H1 headings (`# Title`). All content between
/// H1 headings becomes part of a single slide.
///
/// # Arguments
///
/// * `markdown` - The raw markdown content to parse
/// * `theme_set` - Syntax highlighting themes
/// * `syntax_set` - Syntax definitions for highlighting
/// * `terminal_width` - Width of the terminal for centering H1 headings
///
/// # Returns
///
/// A vector of formatted text, each representing the content of one slide
///
/// # Supported Markdown Features
///
/// - Headings (H1-H6) with proper styling
/// - Paragraphs
/// - Lists (bulleted with •)
/// - Emphasis (*italic*, **bold**) with proper styling
/// - Inline code (`code`) with styling
/// - Code blocks with syntax highlighting (```rust```, ```python```)
fn parse_markdown_to_slides(
    markdown: &str,
    theme_set: &ThemeSet,
    syntax_set: &SyntaxSet,
    terminal_width: u16,
) -> Vec<Text<'static>> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    let parser = MarkdownParser::new_ext(markdown, options);
    let mut slides = Vec::new();
    let mut current_slide_lines: Vec<Line<'static>> = Vec::new();
    let mut current_line_spans: Vec<Span<'static>> = Vec::new();
    let mut in_heading = false;
    let mut heading_level = HeadingLevel::H1;
    let mut in_strong = false;
    let mut in_emphasis = false;
    let mut in_code_block = false;
    let mut code_block_lang: Option<String> = None;
    let mut code_block_content = String::new();
    let mut in_table = false;
    // Stack of list contexts: None = unordered, Some(n) = next number for ordered list
    let mut list_stack: Vec<Option<u64>> = Vec::new();
    let mut table_rows: Vec<Vec<String>> = Vec::new();
    let mut table_header_rows: usize = 0;
    let mut current_table_row: Vec<String> = Vec::new();
    let mut current_cell_content = String::new();

    let theme = &theme_set.themes["base16-ocean.dark"];

    // Inner width of the bordered Paragraph (terminal - 2 for left/right border columns).
    let effective_width: usize = (terminal_width as usize).saturating_sub(2);

    let push_current_line =
        |lines: &mut Vec<Line<'static>>, spans: &mut Vec<Span<'static>>, is_h1: bool| {
            if !spans.is_empty() {
                let mut line = Line::from(std::mem::take(spans));
                if is_h1 {
                    let text_width: usize = line
                        .spans
                        .iter()
                        .map(|span| span.content.as_ref().width())
                        .sum();
                    let padding = effective_width.saturating_sub(text_width) / 2;
                    if padding > 0 {
                        line.spans.insert(0, Span::raw(" ".repeat(padding)));
                    }
                }
                lines.push(line);
            }
        };

    let add_spacing = |lines: &mut Vec<Line<'static>>| {
        if !lines.is_empty() {
            lines.push(Line::from(""));
        }
    };

    let finish_slide = |slides: &mut Vec<Text<'static>>, lines: &mut Vec<Line<'static>>| {
        if !lines.is_empty() {
            slides.push(Text::from(std::mem::take(lines)));
        }
    };

    for event in parser {
        match event {
            MarkdownEvent::Start(Tag::Heading {
                level: HeadingLevel::H1,
                ..
            }) => {
                push_current_line(&mut current_slide_lines, &mut current_line_spans, false);
                if !current_slide_lines.is_empty() {
                    finish_slide(&mut slides, &mut current_slide_lines);
                }
                in_heading = true;
                heading_level = HeadingLevel::H1;
            }
            MarkdownEvent::Start(Tag::Heading { level, .. }) => {
                push_current_line(&mut current_slide_lines, &mut current_line_spans, false);
                in_heading = true;
                heading_level = level;
            }
            MarkdownEvent::End(TagEnd::Heading(_)) => {
                push_current_line(&mut current_slide_lines, &mut current_line_spans, heading_level == HeadingLevel::H1);
                add_spacing(&mut current_slide_lines);
                in_heading = false;
            }
            MarkdownEvent::Text(text) => {
                if in_code_block {
                    code_block_content.push_str(&text);
                } else if in_table {
                    current_cell_content.push_str(&text);
                } else {
                    let mut style = Style::default().fg(Color::White);

                    if in_heading {
                        style = match heading_level {
                            HeadingLevel::H1 => Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                            HeadingLevel::H2 => Style::default()
                                .fg(Color::Blue)
                                .add_modifier(Modifier::BOLD),
                            HeadingLevel::H3 => Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                            _ => Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        };
                    } else if in_strong {
                        style = style.add_modifier(Modifier::BOLD);
                    } else if in_emphasis {
                        style = style.add_modifier(Modifier::ITALIC);
                    }

                    current_line_spans.push(Span::styled(text.to_string(), style));
                }
            }
            MarkdownEvent::Start(Tag::Paragraph) => {
                if !in_table {
                    push_current_line(&mut current_slide_lines, &mut current_line_spans, false);
                }
            }
            MarkdownEvent::End(TagEnd::Paragraph) => {
                if !in_table {
                    push_current_line(&mut current_slide_lines, &mut current_line_spans, false);
                    add_spacing(&mut current_slide_lines);
                }
            }
            MarkdownEvent::Start(Tag::List(start)) => {
                push_current_line(&mut current_slide_lines, &mut current_line_spans, false);
                list_stack.push(start);
            }
            MarkdownEvent::Start(Tag::Item) => {
                // Indent nested list items by two spaces per nesting level past the first.
                let depth = list_stack.len().saturating_sub(1);
                if depth > 0 {
                    current_line_spans.push(Span::raw(" ".repeat(depth * 2)));
                }
                let marker = match list_stack.last_mut() {
                    Some(Some(n)) => {
                        let marker = format!("{}. ", *n);
                        *n += 1;
                        marker
                    }
                    _ => "• ".to_string(),
                };
                current_line_spans
                    .push(Span::styled(marker, Style::default().fg(Color::Yellow)));
            }
            MarkdownEvent::End(TagEnd::Item) => {
                push_current_line(&mut current_slide_lines, &mut current_line_spans, false);
            }
            MarkdownEvent::End(TagEnd::List(_)) => {
                list_stack.pop();
                if list_stack.is_empty() {
                    add_spacing(&mut current_slide_lines);
                }
            }
            MarkdownEvent::Start(Tag::Strong) => {
                in_strong = true;
            }
            MarkdownEvent::End(TagEnd::Strong) => {
                in_strong = false;
            }
            MarkdownEvent::Start(Tag::Emphasis) => {
                in_emphasis = true;
            }
            MarkdownEvent::End(TagEnd::Emphasis) => {
                in_emphasis = false;
            }
            MarkdownEvent::Code(code) => {
                if in_table {
                    current_cell_content.push_str(&format!("`{}`", code));
                } else {
                    current_line_spans.push(Span::styled(
                        format!("`{}`", code),
                        Style::default().fg(Color::Green).bg(Color::Rgb(40, 40, 40)),
                    ));
                }
            }
            MarkdownEvent::Start(Tag::CodeBlock(info)) => {
                push_current_line(&mut current_slide_lines, &mut current_line_spans, false);
                in_code_block = true;
                code_block_lang = match info {
                    pulldown_cmark::CodeBlockKind::Indented => None,
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                        if lang.is_empty() {
                            None
                        } else {
                            Some(lang.to_string())
                        }
                    }
                };
                code_block_content.clear();
            }
            MarkdownEvent::End(TagEnd::CodeBlock) => {
                in_code_block = false;

                let lang_lower = code_block_lang.as_deref().map(|s| s.to_lowercase());
                let is_lean = matches!(lang_lower.as_deref(), Some("lean") | Some("lean4"));

                if is_lean {
                    for line in highlight_lean4_code(&code_block_content) {
                        current_slide_lines.push(line);
                    }
                } else if let Some(lang) = &code_block_lang {
                    // Try to find syntax by the language name first, then by common extensions
                    let syntax = syntax_set.find_syntax_by_token(lang)
                        .or_else(|| {
                            // Map common language names to their file extensions
                            let extension = match lang.as_str() {
                                "rust" | "rs" => "rs",
                                "python" | "py" => "py",
                                "javascript" | "js" => "js",
                                "typescript" | "ts" => "ts",
                                "java" => "java",
                                "c" => "c",
                                "cpp" | "c++" | "cxx" => "cpp",
                                "csharp" | "c#" | "cs" => "cs",
                                "go" | "golang" => "go",
                                "html" => "html",
                                "css" => "css",
                                "json" => "json",
                                "xml" => "xml",
                                "yaml" | "yml" => "yaml",
                                "toml" => "toml",
                                "markdown" | "md" => "md",
                                "dockerfile" | "docker" => "Dockerfile",
                                "sql" => "sql",
                                "shell" | "bash" | "sh" => "sh",
                                "php" => "php",
                                "ruby" | "rb" => "rb",
                                "perl" | "pl" => "pl",
                                "swift" => "swift",
                                "kotlin" | "kt" => "kt",
                                "scala" => "scala",
                                "haskell" | "hs" => "hs",
                                "elixir" | "ex" => "ex",
                                "erlang" | "erl" => "erl",
                                "clojure" | "clj" => "clj",
                                "lua" => "lua",
                                "r" => "r",
                                "matlab" => "m",
                                "powershell" | "ps1" => "ps1",
                                "vim" => "vim",
                                "tex" | "latex" => "tex",
                                "makefile" | "make" => "Makefile",
                                "nginx" => "conf",
                                "apache" => "conf",
                                "ini" => "ini",
                                "properties" => "properties",
                                "groovy" => "groovy",
                                "dart" => "dart",
                                "assembly" | "asm" => "asm",
                                "lisp" => "lisp",
                                "scheme" => "scm",
                                "ocaml" => "ml",
                                "fsharp" | "f#" => "fs",
                                "pascal" => "pas",
                                "fortran" => "f90",
                                "cobol" => "cob",
                                "ada" => "ada",
                                "verilog" => "v",
                                "vhdl" => "vhd",
                                _ => lang, // Fall back to using the language name as extension
                            };
                            syntax_set.find_syntax_by_extension(extension)
                        });
                    
                    if let Some(syntax) = syntax {
                        let mut highlighter = HighlightLines::new(syntax, theme);

                        for line in LinesWithEndings::from(&code_block_content) {
                            // syntect uses the trailing \n for context, but the \n must not
                            // leak into ratatui spans (it would be rendered as a control char).
                            let ranges = highlighter
                                .highlight_line(line, syntax_set)
                                .unwrap_or_default();
                            let mut line_spans = Vec::new();

                            if ranges.is_empty() {
                                let clean = line.trim_end_matches(['\n', '\r']).to_string();
                                line_spans.push(Span::styled(
                                    clean,
                                    Style::default().fg(Color::Green),
                                ));
                            } else {
                                for (style, text) in ranges {
                                    let clean = text.trim_end_matches(['\n', '\r']);
                                    if clean.is_empty() {
                                        continue;
                                    }
                                    let fg_color = Color::Rgb(
                                        style.foreground.r,
                                        style.foreground.g,
                                        style.foreground.b,
                                    );
                                    let mut ratatui_style = Style::default().fg(fg_color);

                                    if style
                                        .font_style
                                        .contains(syntect::highlighting::FontStyle::BOLD)
                                    {
                                        ratatui_style = ratatui_style.add_modifier(Modifier::BOLD);
                                    }
                                    if style
                                        .font_style
                                        .contains(syntect::highlighting::FontStyle::ITALIC)
                                    {
                                        ratatui_style =
                                            ratatui_style.add_modifier(Modifier::ITALIC);
                                    }

                                    line_spans.push(Span::styled(clean.to_string(), ratatui_style));
                                }
                            }

                            current_slide_lines.push(Line::from(line_spans));
                        }
                    } else {
                        // Fallback to unstyled code if no syntax is found
                        for line in code_block_content.lines() {
                            current_slide_lines.push(Line::from(Span::styled(
                                line.to_string(),
                                Style::default().fg(Color::Green),
                            )));
                        }
                    }
                } else {
                    for line in code_block_content.lines() {
                        current_slide_lines.push(Line::from(Span::styled(
                            line.to_string(),
                            Style::default().fg(Color::Green),
                        )));
                    }
                }

                code_block_content.clear();
                code_block_lang = None;
                add_spacing(&mut current_slide_lines);
            }
            MarkdownEvent::Start(Tag::Table(_)) => {
                push_current_line(&mut current_slide_lines, &mut current_line_spans, false);
                in_table = true;
                table_rows.clear();
                table_header_rows = 0;
            }
            MarkdownEvent::End(TagEnd::Table) => {
                // Render the complete table
                if !table_rows.is_empty() {
                    // Calculate column widths
                    let num_cols = table_rows.iter().map(|row| row.len()).max().unwrap_or(0);
                    let mut col_widths = vec![0; num_cols];
                    
                    for row in &table_rows {
                        for (i, cell) in row.iter().enumerate() {
                            if i < col_widths.len() {
                                col_widths[i] = col_widths[i].max(cell.width());
                            }
                        }
                    }
                    
                    // Add top border
                    let mut top_border_spans = Vec::new();
                    top_border_spans.push(Span::styled("┌", Style::default().fg(Color::Gray)));
                    for (i, width) in col_widths.iter().enumerate() {
                        top_border_spans.push(Span::styled("─".repeat(width + 2), Style::default().fg(Color::Gray)));
                        if i < col_widths.len() - 1 {
                            top_border_spans.push(Span::styled("┬", Style::default().fg(Color::Gray)));
                        }
                    }
                    top_border_spans.push(Span::styled("┐", Style::default().fg(Color::Gray)));
                    current_slide_lines.push(Line::from(top_border_spans));
                    
                    // Render table rows
                    for (row_idx, row) in table_rows.iter().enumerate() {
                        let is_header_row = row_idx < table_header_rows;
                        let cell_style = if is_header_row {
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::White)
                        };

                        let mut line_spans = Vec::new();
                        line_spans.push(Span::styled("│ ", Style::default().fg(Color::Gray)));

                        for (col_idx, cell) in row.iter().enumerate() {
                            let width = col_widths.get(col_idx).unwrap_or(&10);
                            let cell_width = cell.width();
                            let padding_needed = width.saturating_sub(cell_width);
                            let padded_cell = format!("{}{}", cell, " ".repeat(padding_needed));

                            line_spans.push(Span::styled(padded_cell, cell_style));
                            line_spans.push(Span::styled(" │ ", Style::default().fg(Color::Gray)));
                        }

                        current_slide_lines.push(Line::from(line_spans));

                        // Use a heavier separator after the header row, lighter between body rows.
                        if row_idx < table_rows.len() - 1 {
                            let is_header_boundary = row_idx + 1 == table_header_rows;
                            let mut sep_spans = Vec::new();
                            if is_header_boundary {
                                sep_spans
                                    .push(Span::styled("╞", Style::default().fg(Color::Gray)));
                                for (i, width) in col_widths.iter().enumerate() {
                                    sep_spans.push(Span::styled(
                                        "═".repeat(width + 2),
                                        Style::default().fg(Color::Gray),
                                    ));
                                    if i < col_widths.len() - 1 {
                                        sep_spans.push(Span::styled(
                                            "╪",
                                            Style::default().fg(Color::Gray),
                                        ));
                                    }
                                }
                                sep_spans
                                    .push(Span::styled("╡", Style::default().fg(Color::Gray)));
                            } else {
                                sep_spans
                                    .push(Span::styled("├", Style::default().fg(Color::Gray)));
                                for (i, width) in col_widths.iter().enumerate() {
                                    sep_spans.push(Span::styled(
                                        "─".repeat(width + 2),
                                        Style::default().fg(Color::Gray),
                                    ));
                                    if i < col_widths.len() - 1 {
                                        sep_spans.push(Span::styled(
                                            "┼",
                                            Style::default().fg(Color::Gray),
                                        ));
                                    }
                                }
                                sep_spans
                                    .push(Span::styled("┤", Style::default().fg(Color::Gray)));
                            }
                            current_slide_lines.push(Line::from(sep_spans));
                        }
                    }
                    
                    // Add bottom border
                    let mut bottom_border_spans = Vec::new();
                    bottom_border_spans.push(Span::styled("└", Style::default().fg(Color::Gray)));
                    for (i, width) in col_widths.iter().enumerate() {
                        bottom_border_spans.push(Span::styled("─".repeat(width + 2), Style::default().fg(Color::Gray)));
                        if i < col_widths.len() - 1 {
                            bottom_border_spans.push(Span::styled("┴", Style::default().fg(Color::Gray)));
                        }
                    }
                    bottom_border_spans.push(Span::styled("┘", Style::default().fg(Color::Gray)));
                    current_slide_lines.push(Line::from(bottom_border_spans));
                }
                
                add_spacing(&mut current_slide_lines);
                in_table = false;
            }
            MarkdownEvent::Start(Tag::TableHead) => {}
            MarkdownEvent::End(TagEnd::TableHead) => {
                table_header_rows = table_rows.len();
            }
            MarkdownEvent::Start(Tag::TableRow) => {
                current_table_row.clear();
            }
            MarkdownEvent::End(TagEnd::TableRow) => {
                table_rows.push(current_table_row.clone());
                current_table_row.clear();
            }
            MarkdownEvent::Start(Tag::TableCell) => {
                current_cell_content.clear();
            }
            MarkdownEvent::End(TagEnd::TableCell) => {
                current_table_row.push(current_cell_content.trim().to_string());
                current_cell_content.clear();
            }
            MarkdownEvent::SoftBreak | MarkdownEvent::HardBreak => {
                if !in_table {
                    push_current_line(&mut current_slide_lines, &mut current_line_spans, false);
                }
            }
            MarkdownEvent::Rule => {
                push_current_line(&mut current_slide_lines, &mut current_line_spans, false);
                // Render a horizontal rule as a line of dashes spanning the inner width.
                let rule_width = effective_width.max(4);
                current_slide_lines.push(Line::from(Span::styled(
                    "─".repeat(rule_width),
                    Style::default().fg(Color::DarkGray),
                )));
                add_spacing(&mut current_slide_lines);
            }
            _ => {}
        }
    }

    push_current_line(&mut current_slide_lines, &mut current_line_spans, false);
    finish_slide(&mut slides, &mut current_slide_lines);

    if slides.is_empty() {
        slides.push(Text::from("No slides found in markdown file"));
    }

    slides
}

/// Lean 4 keywords — declarations, modifiers, and structural forms.
const LEAN4_KEYWORDS: &[&str] = &[
    "def", "theorem", "lemma", "example", "instance", "class", "structure",
    "inductive", "coinductive", "abbrev", "axiom", "constant", "opaque",
    "namespace", "section", "end", "open", "import", "export", "universe",
    "universes", "variable", "variables", "notation", "infix", "infixl",
    "infixr", "prefix", "postfix", "syntax", "macro", "macro_rules", "elab",
    "elab_rules", "builtin_initialize", "initialize", "deriving", "extends",
    "mutual", "where", "do", "if", "then", "else", "match", "with", "let",
    "in", "fun", "λ", "have", "show", "from", "suffices", "calc", "return",
    "unless", "for", "while", "try", "catch", "finally", "throw", "break",
    "continue", "at", "by",
    "private", "protected", "partial", "unsafe", "noncomputable", "nonrec",
    "scoped", "local", "set_option", "attribute", "@[simp]", "#check",
    "#eval", "#print", "#reduce",
];

/// Lean 4 tactics commonly seen inside `by` blocks.
const LEAN4_TACTICS: &[&str] = &[
    "rfl", "simp", "simp_all", "simp_rw", "rw", "exact", "apply", "intro",
    "intros", "constructor", "induction", "cases", "rcases", "rintro",
    "obtain", "use", "refine", "refine'", "tauto", "aesop", "omega",
    "linarith", "nlinarith", "polyrith", "positivity", "ring", "ring_nf",
    "field_simp", "norm_num", "norm_cast", "push_cast", "decide", "trivial",
    "assumption", "contradiction", "sorry", "admit", "change", "split",
    "left", "right", "unfold", "symm", "trans", "ext", "funext", "propext",
    "push_neg", "specialize", "exact?", "apply?", "hint", "conv", "skip",
    "first", "all_goals", "any_goals", "repeat", "iterate", "solve",
    "solve_by_elim", "fin_cases", "interval_cases", "choose", "subst",
    "subst_vars", "clear", "rename_i", "rename", "revert", "generalize",
    "nlinarith", "done",
];

/// Lean 4 built-in types and Sort-family keywords.
const LEAN4_TYPES: &[&str] = &[
    "Prop", "Type", "Sort", "Nat", "Int", "Rat", "Real", "Bool", "String",
    "Char", "List", "Array", "Option", "Unit", "Empty", "Fin", "Set",
    "Subtype", "Sum", "Prod", "Sigma", "PSigma", "IO",
];

fn lean4_style(word: &str) -> Option<Style> {
    if LEAN4_KEYWORDS.contains(&word) {
        Some(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )
    } else if LEAN4_TACTICS.contains(&word) {
        Some(Style::default().fg(Color::LightBlue))
    } else if LEAN4_TYPES.contains(&word) {
        Some(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        None
    }
}

/// Highlights a Lean 4 source code string into per-line styled spans.
///
/// Handles line comments (`-- ...`), nested block comments (`/- ... -/`),
/// string literals, numeric literals, attribute forms like `@[simp]`, common
/// unicode operators (∀, ∃, λ, →, ↔, ∧, ∨, etc.), keywords, tactics, and
/// built-in types. Anything else is emitted with the default foreground.
fn highlight_lean4_code(content: &str) -> Vec<Line<'static>> {
    let comment_style = Style::default()
        .fg(Color::DarkGray)
        .add_modifier(Modifier::ITALIC);
    let string_style = Style::default().fg(Color::LightGreen);
    let number_style = Style::default().fg(Color::LightYellow);
    let attribute_style = Style::default().fg(Color::LightMagenta);
    let operator_style = Style::default()
        .fg(Color::Magenta)
        .add_modifier(Modifier::BOLD);
    let default_style = Style::default().fg(Color::White);

    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut block_comment_depth: u32 = 0;

    for line_text in content.split('\n') {
        let chars: Vec<char> = line_text.chars().collect();
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut i = 0;

        // If we carried a block comment from the previous line, continue consuming it.
        if block_comment_depth > 0 {
            let start = i;
            while i < chars.len() && block_comment_depth > 0 {
                if chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '-' {
                    block_comment_depth += 1;
                    i += 2;
                } else if chars[i] == '-' && i + 1 < chars.len() && chars[i + 1] == '/' {
                    block_comment_depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            if i > start {
                let s: String = chars[start..i].iter().collect();
                spans.push(Span::styled(s, comment_style));
            }
        }

        while i < chars.len() {
            let c = chars[i];

            // Line comment: `-- ...` to end of line.
            if c == '-' && i + 1 < chars.len() && chars[i + 1] == '-' {
                let s: String = chars[i..].iter().collect();
                spans.push(Span::styled(s, comment_style));
                break;
            }

            // Block comment: `/- ... -/` (possibly nested, possibly multi-line).
            if c == '/' && i + 1 < chars.len() && chars[i + 1] == '-' {
                let start = i;
                block_comment_depth = 1;
                i += 2;
                while i < chars.len() && block_comment_depth > 0 {
                    if chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '-' {
                        block_comment_depth += 1;
                        i += 2;
                    } else if chars[i] == '-' && i + 1 < chars.len() && chars[i + 1] == '/' {
                        block_comment_depth -= 1;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                let s: String = chars[start..i].iter().collect();
                spans.push(Span::styled(s, comment_style));
                continue;
            }

            // String literal.
            if c == '"' {
                let start = i;
                i += 1;
                while i < chars.len() {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 2;
                        continue;
                    }
                    if chars[i] == '"' {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                spans.push(Span::styled(s, string_style));
                continue;
            }

            // Char literal: `'a'`, `'\n'`, etc. (bounded, safe to treat as string-colored).
            if c == '\'' && i + 1 < chars.len() {
                let start = i;
                i += 1;
                if chars[i] == '\\' && i + 1 < chars.len() {
                    i += 2;
                } else {
                    i += 1;
                }
                if i < chars.len() && chars[i] == '\'' {
                    i += 1;
                    let s: String = chars[start..i].iter().collect();
                    spans.push(Span::styled(s, string_style));
                    continue;
                }
                // Not a char literal — rewind and treat as default.
                i = start;
                let mut s = String::new();
                s.push(chars[i]);
                spans.push(Span::styled(s, default_style));
                i += 1;
                continue;
            }

            // Attribute form `@[...]`.
            if c == '@' && i + 1 < chars.len() && chars[i + 1] == '[' {
                let start = i;
                i += 2;
                while i < chars.len() && chars[i] != ']' {
                    i += 1;
                }
                if i < chars.len() {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                spans.push(Span::styled(s, attribute_style));
                continue;
            }

            // Numeric literal.
            if c.is_ascii_digit() {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '_') {
                    i += 1;
                }
                if i < chars.len() && chars[i] == '.'
                    && i + 1 < chars.len()
                    && chars[i + 1].is_ascii_digit()
                {
                    i += 1;
                    while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '_') {
                        i += 1;
                    }
                }
                let s: String = chars[start..i].iter().collect();
                spans.push(Span::styled(s, number_style));
                continue;
            }

            // Identifier / keyword.
            if c.is_alphabetic() || c == '_' {
                let start = i;
                while i < chars.len()
                    && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '\'')
                {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                let style = lean4_style(word.as_str()).unwrap_or(default_style);
                spans.push(Span::styled(word, style));
                continue;
            }

            // Common unicode operators used in Lean 4.
            if matches!(
                c,
                '∀' | '∃'
                    | 'λ'
                    | '→'
                    | '←'
                    | '↔'
                    | '⇒'
                    | '∧'
                    | '∨'
                    | '¬'
                    | '≤'
                    | '≥'
                    | '≠'
                    | '≡'
                    | '∈'
                    | '∉'
                    | '⊆'
                    | '⊂'
                    | '∪'
                    | '∩'
                    | '⟨'
                    | '⟩'
                    | '⊢'
                    | '⊤'
                    | '⊥'
                    | '∘'
                    | '∅'
                    | '×'
            ) {
                let mut s = String::new();
                s.push(c);
                spans.push(Span::styled(s, operator_style));
                i += 1;
                continue;
            }

            // Default character.
            let mut s = String::new();
            s.push(c);
            spans.push(Span::styled(s, default_style));
            i += 1;
        }

        lines.push(Line::from(spans));
    }

    lines
}

/// Renders the user interface for the slideshow.
///
/// Creates a two-panel layout with the main slide content on top
/// and navigation information at the bottom.
///
/// # Arguments
///
/// * `f` - The frame to render into
/// * `app` - The application state containing slide data
fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

    let slide_content = app.current_slide_content();
    
    // Apply scroll offset to the content
    let visible_lines: Vec<_> = slide_content
        .lines
        .iter()
        .skip(app.scroll_offset)
        .cloned()
        .collect();
    
    let scrolled_content = Text::from(visible_lines);
    
    let paragraph = Paragraph::new(scrolled_content)
        .block(
            Block::default()
                .title("Markdown Slideshow")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, chunks[0]);

    let info_text = format!(
        " Slide {} | ← → Navigate | ↑ ↓ Scroll | Home/End First/Last | q Quit ",
        app.slide_info()
    );
    let info = Paragraph::new(info_text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Yellow));

    f.render_widget(info, chunks[1]);
}

/// Main application loop that handles user input and rendering.
///
/// Continuously draws the UI and processes keyboard events until
/// the user quits the application.
///
/// # Arguments
///
/// * `terminal` - The terminal instance to draw to
/// * `app` - The application state to manage
///
/// # Returns
///
/// Result indicating success or I/O error
///
/// # Keyboard Controls
///
/// - `q`, `Esc`: Quit the application
/// - `→`, `l`, `Space`: Next slide
/// - `←`, `h`: Previous slide
/// - `↑`, `k`: Scroll up within slide
/// - `↓`, `j`: Scroll down within slide
fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        match event::read()? {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Right | KeyCode::Char('l') | KeyCode::Char(' ') => app.next_slide(),
                KeyCode::Left | KeyCode::Char('h') => app.prev_slide(),
                KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                KeyCode::PageDown => app.next_slide(),
                KeyCode::PageUp => app.prev_slide(),
                KeyCode::Home => app.goto_first(),
                KeyCode::End => app.goto_last(),
                _ => {}
            },
            Event::Resize(w, _) => app.resize(w),
            _ => {}
        }
    }
    Ok(())
}

/// Main entry point for the presentation application.
///
/// Parses command line arguments, sets up the terminal, runs the slideshow,
/// and cleans up the terminal state before exiting.
///
/// # Returns
///
/// Result indicating success or error during execution
///
/// # Errors
///
/// - File I/O errors when reading the markdown file
/// - Terminal setup/cleanup errors
/// - Application runtime errors
fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let markdown_content = fs::read_to_string(&args.file)
        .map_err(|e| format!("Failed to read file '{}': {}", args.file, e))?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let terminal_size = terminal.size()?;
    let app = App::new(markdown_content, terminal_size.width);
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}
