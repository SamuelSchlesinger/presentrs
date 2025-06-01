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
    ///
    /// # Arguments
    ///
    /// * `markdown_content` - The raw markdown content to parse into slides
    ///
    /// # Returns
    ///
    /// A new App instance with slides parsed from the markdown content
    fn new(markdown_content: &str, terminal_width: u16) -> Self {
        let theme_set = ThemeSet::load_defaults();
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let slides = parse_markdown_to_slides(markdown_content, &theme_set, &syntax_set, terminal_width);
        App {
            slides,
            current_slide: 0,
            scroll_offset: 0,
            theme_set,
            syntax_set,
        }
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
    let mut in_list = false;
    let mut table_rows: Vec<Vec<String>> = Vec::new();
    let mut current_table_row: Vec<String> = Vec::new();
    let mut current_cell_content = String::new();
    let mut _in_table_header = false;

    let theme = &theme_set.themes["base16-ocean.dark"];

    let push_current_line = |lines: &mut Vec<Line<'static>>, spans: &mut Vec<Span<'static>>, is_h1: bool| {
        if !spans.is_empty() {
            let mut line = Line::from(std::mem::take(spans));
            if is_h1 {
                // Center the H1 line by calculating padding
                let text_width: usize = line.spans.iter()
                    .map(|span| span.content.chars().count())
                    .sum();
                let padding = if terminal_width as usize > text_width {
                    (terminal_width as usize - text_width) / 2
                } else {
                    0
                };
                
                if padding > 0 {
                    let padding_span = Span::raw(" ".repeat(padding));
                    line.spans.insert(0, padding_span);
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
            MarkdownEvent::Start(Tag::List(_)) => {
                push_current_line(&mut current_slide_lines, &mut current_line_spans, false);
                in_list = true;
            }
            MarkdownEvent::Start(Tag::Item) => {
                current_line_spans.push(Span::styled("• ", Style::default().fg(Color::Yellow)));
            }
            MarkdownEvent::End(TagEnd::Item) => {
                push_current_line(&mut current_slide_lines, &mut current_line_spans, false);
            }
            MarkdownEvent::End(TagEnd::List(_)) => {
                if in_list {
                    add_spacing(&mut current_slide_lines);
                    in_list = false;
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

                if let Some(lang) = &code_block_lang {
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
                            let ranges = highlighter
                                .highlight_line(line, syntax_set)
                                .unwrap_or_default();
                            let mut line_spans = Vec::new();

                            // If highlighting fails or produces no ranges, preserve the original line
                            if ranges.is_empty() {
                                line_spans.push(Span::styled(
                                    line.to_string(),
                                    Style::default().fg(Color::Green),
                                ));
                            } else {
                                for (style, text) in ranges {
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

                                    // Preserve the exact text including whitespace
                                    line_spans.push(Span::styled(text.to_string(), ratatui_style));
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
                        let mut line_spans = Vec::new();
                        line_spans.push(Span::styled("│ ", Style::default().fg(Color::Gray)));
                        
                        for (col_idx, cell) in row.iter().enumerate() {
                            let width = col_widths.get(col_idx).unwrap_or(&10);
                            let cell_width = cell.width();
                            let padding_needed = width.saturating_sub(cell_width);
                            let padded_cell = format!("{}{}", cell, " ".repeat(padding_needed));
                            
                            line_spans.push(Span::styled(padded_cell, Style::default().fg(Color::White)));
                            line_spans.push(Span::styled(" │ ", Style::default().fg(Color::Gray)));
                        }
                        
                        current_slide_lines.push(Line::from(line_spans));
                        
                        // Add separator line between all rows (except after the last row)
                        if row_idx < table_rows.len() - 1 {
                            let mut sep_spans = Vec::new();
                            sep_spans.push(Span::styled("├", Style::default().fg(Color::Gray)));
                            for (i, width) in col_widths.iter().enumerate() {
                                sep_spans.push(Span::styled("─".repeat(width + 2), Style::default().fg(Color::Gray)));
                                if i < col_widths.len() - 1 {
                                    sep_spans.push(Span::styled("┼", Style::default().fg(Color::Gray)));
                                }
                            }
                            sep_spans.push(Span::styled("┤", Style::default().fg(Color::Gray)));
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
                _in_table_header = false;
            }
            MarkdownEvent::Start(Tag::TableHead) => {
                _in_table_header = true;
            }
            MarkdownEvent::End(TagEnd::TableHead) => {
                _in_table_header = false;
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

    let info_text = format!(" Slide {} | ← → Navigate | ↑ ↓ Scroll | q Quit ", app.slide_info());
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

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Right | KeyCode::Char('l') | KeyCode::Char(' ') => app.next_slide(),
                KeyCode::Left | KeyCode::Char('h') => app.prev_slide(),
                KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                _ => {}
            }
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
    let app = App::new(&markdown_content, terminal_size.width);
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
