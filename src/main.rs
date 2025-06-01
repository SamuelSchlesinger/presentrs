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
    #[allow(dead_code)]
    theme_set: ThemeSet,
    /// Syntax definitions
    #[allow(dead_code)]
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
    fn new(markdown_content: &str) -> Self {
        let theme_set = ThemeSet::load_defaults();
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let slides = parse_markdown_to_slides(markdown_content, &theme_set, &syntax_set);
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
    /// Does nothing if already on the last slide.
    fn next_slide(&mut self) {
        if self.current_slide < self.slides.len() - 1 {
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
        let max_scroll = self.slides[self.current_slide].lines.len().saturating_sub(1);
        if self.scroll_offset < max_scroll {
            self.scroll_offset += 1;
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
    /// A reference to the current slide's content
    fn current_slide_content(&self) -> &Text<'static> {
        &self.slides[self.current_slide]
    }

    /// Returns a formatted string showing current slide position.
    ///
    /// # Returns
    ///
    /// A string in the format "current/total" (e.g., "3/10")
    fn slide_info(&self) -> String {
        format!("{}/{}", self.current_slide + 1, self.slides.len())
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

    let theme = &theme_set.themes["base16-ocean.dark"];

    let push_current_line = |lines: &mut Vec<Line<'static>>, spans: &mut Vec<Span<'static>>, is_h1: bool| {
        if !spans.is_empty() {
            let mut line = Line::from(std::mem::take(spans));
            if is_h1 {
                // Center the H1 line by calculating padding
                let text_width: usize = line.spans.iter()
                    .map(|span| span.content.chars().count())
                    .sum();
                let terminal_width = 80; // Assume 80 chars width, could be made dynamic
                let padding = if terminal_width > text_width {
                    (terminal_width - text_width) / 2
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
                in_heading = false;
            }
            MarkdownEvent::Text(text) => {
                if in_code_block {
                    code_block_content.push_str(&text);
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
                    } else if in_table {
                        style = Style::default().fg(Color::Cyan); // Light blue for table content
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
                }
            }
            MarkdownEvent::Start(Tag::List(_)) => {
                push_current_line(&mut current_slide_lines, &mut current_line_spans, false);
            }
            MarkdownEvent::Start(Tag::Item) => {
                current_line_spans.push(Span::styled("• ", Style::default().fg(Color::Yellow)));
            }
            MarkdownEvent::End(TagEnd::Item) => {
                push_current_line(&mut current_slide_lines, &mut current_line_spans, false);
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
                current_line_spans.push(Span::styled(
                    format!("`{}`", code),
                    Style::default().fg(Color::Green).bg(Color::Rgb(40, 40, 40)),
                ));
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
                    let syntax_extension = match lang.as_str() {
                        "rust" => Some("rs"),
                        "python" | "py" => Some("py"),
                        _ => None,
                    };
                    
                    if let Some(ext) = syntax_extension {
                        if let Some(syntax) = syntax_set.find_syntax_by_extension(ext) {
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
            }
            MarkdownEvent::Start(Tag::Table(_)) => {
                push_current_line(&mut current_slide_lines, &mut current_line_spans, false);
                in_table = true;
            }
            MarkdownEvent::End(TagEnd::Table) => {
                push_current_line(&mut current_slide_lines, &mut current_line_spans, false);
                in_table = false;
            }
            MarkdownEvent::Start(Tag::TableHead) => {
                // Table header - no special handling needed
            }
            MarkdownEvent::End(TagEnd::TableHead) => {
                push_current_line(&mut current_slide_lines, &mut current_line_spans, false);
            }
            MarkdownEvent::Start(Tag::TableRow) => {
                // Start new row
            }
            MarkdownEvent::End(TagEnd::TableRow) => {
                push_current_line(&mut current_slide_lines, &mut current_line_spans, false);
            }
            MarkdownEvent::Start(Tag::TableCell) => {
                // Add some spacing between cells
                if !current_line_spans.is_empty() {
                    current_line_spans.push(Span::styled(" | ", Style::default().fg(Color::Gray)));
                }
            }
            MarkdownEvent::End(TagEnd::TableCell) => {
                // Cell content already added via Text events
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

    let app = App::new(&markdown_content);
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
