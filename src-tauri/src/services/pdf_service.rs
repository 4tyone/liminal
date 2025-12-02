use pulldown_cmark::{Parser, Event, Tag, TagEnd, CodeBlockKind};
use headless_chrome::{Browser, LaunchOptions, types::PrintToPdfOptions};
use std::fs;
use std::time::Duration;

const WEBSITE_URL: &str = "https://liminal.wrappt.tech";
const WATERMARK_TEXT: &str = "Customize your learning with Liminal";

/// Generate a PDF document with embedded CSS that matches the app's styling
pub fn export_project_to_pdf(
    title: &str,
    pages: Vec<String>,
    output_path: &str,
) -> Result<(), String> {
    let mut html_content = String::new();

    // Process each page's markdown to HTML
    for (idx, markdown) in pages.iter().enumerate() {
        if idx > 0 {
            html_content.push_str(r#"<div class="page-break"></div>"#);
        }
        let page_html = markdown_to_html(markdown);
        html_content.push_str(&format!(r#"<section class="chapter">{}</section>"#, page_html));
    }

    let full_html = generate_full_html(title, &html_content);

    // Write HTML to a temporary file (data URLs have size limits)
    let temp_dir = std::env::temp_dir();
    let temp_html_path = temp_dir.join("liminal_export.html");
    fs::write(&temp_html_path, &full_html)
        .map_err(|e| format!("Failed to write temporary HTML: {}", e))?;

    let file_url = format!("file://{}", temp_html_path.to_string_lossy());

    // Use headless Chrome to generate PDF
    let browser = Browser::new(
        LaunchOptions::default_builder()
            .headless(true)
            .build()
            .map_err(|e| format!("Failed to build launch options: {}", e))?,
    )
    .map_err(|e| format!("Failed to launch browser: {}", e))?;

    let tab = browser.new_tab()
        .map_err(|e| format!("Failed to create tab: {}", e))?;

    // Navigate to file URL
    tab.navigate_to(&file_url)
        .map_err(|e| format!("Failed to navigate: {}", e))?;

    tab.wait_until_navigated()
        .map_err(|e| format!("Failed to wait for navigation: {}", e))?;

    // Wait a bit for fonts and highlight.js to load
    std::thread::sleep(Duration::from_millis(1500));

    // Generate PDF with options
    let pdf_options = PrintToPdfOptions {
        landscape: Some(false),
        display_header_footer: Some(false),
        print_background: Some(true),
        scale: Some(1.0),
        paper_width: Some(8.27),  // A4 width in inches
        paper_height: Some(11.69), // A4 height in inches
        margin_top: Some(0.4),
        margin_bottom: Some(0.6),
        margin_left: Some(0.4),
        margin_right: Some(0.4),
        page_ranges: None,
        ignore_invalid_page_ranges: None,
        header_template: None,
        footer_template: None,
        prefer_css_page_size: Some(true),
        transfer_mode: None,
        generate_tagged_pdf: None,
        generate_document_outline: None,
    };

    let pdf_data = tab.print_to_pdf(Some(pdf_options))
        .map_err(|e| format!("Failed to generate PDF: {}", e))?;

    // Clean up temporary file
    let _ = fs::remove_file(&temp_html_path);

    fs::write(output_path, pdf_data)
        .map_err(|e| format!("Failed to write PDF: {}", e))?;

    Ok(())
}

fn markdown_to_html(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut html = String::new();
    let mut in_code_block = false;
    let mut code_lang = String::new();
    let mut code_content = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                html.push_str(&format!("<h{}>", level as u8));
            }
            Event::End(TagEnd::Heading(level)) => {
                html.push_str(&format!("</h{}>", level as u8));
            }
            Event::Start(Tag::Paragraph) => {
                html.push_str("<p>");
            }
            Event::End(TagEnd::Paragraph) => {
                html.push_str("</p>");
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                code_lang = match kind {
                    CodeBlockKind::Fenced(lang) => lang.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                code_content.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                let lang_class = if code_lang.is_empty() {
                    "plaintext".to_string()
                } else {
                    code_lang.clone()
                };
                html.push_str(&format!(
                    r#"<pre><code class="language-{}">{}</code></pre>"#,
                    lang_class,
                    html_escape(&code_content)
                ));
            }
            Event::Start(Tag::List(None)) => {
                html.push_str("<ul>");
            }
            Event::End(TagEnd::List(false)) => {
                html.push_str("</ul>");
            }
            Event::Start(Tag::List(Some(_))) => {
                html.push_str("<ol>");
            }
            Event::End(TagEnd::List(true)) => {
                html.push_str("</ol>");
            }
            Event::Start(Tag::Item) => {
                html.push_str("<li>");
            }
            Event::End(TagEnd::Item) => {
                html.push_str("</li>");
            }
            Event::Start(Tag::BlockQuote(_)) => {
                html.push_str("<blockquote>");
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                html.push_str("</blockquote>");
            }
            Event::Start(Tag::Strong) => {
                html.push_str("<strong>");
            }
            Event::End(TagEnd::Strong) => {
                html.push_str("</strong>");
            }
            Event::Start(Tag::Emphasis) => {
                html.push_str("<em>");
            }
            Event::End(TagEnd::Emphasis) => {
                html.push_str("</em>");
            }
            Event::Code(text) => {
                html.push_str(&format!("<code>{}</code>", html_escape(&text)));
            }
            Event::Text(text) => {
                if in_code_block {
                    code_content.push_str(&text);
                } else {
                    html.push_str(&html_escape(&text));
                }
            }
            Event::SoftBreak => {
                if in_code_block {
                    code_content.push('\n');
                } else {
                    html.push(' ');
                }
            }
            Event::HardBreak => {
                html.push_str("<br>");
            }
            Event::Rule => {
                html.push_str("<hr>");
            }
            _ => {}
        }
    }

    html
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn generate_full_html(title: &str, content: &str) -> String {
    format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title} - Liminal</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Libre+Baskerville:ital,wght@0,400;0,700;1,400&display=swap" rel="stylesheet">
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github-dark.min.css">
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js"></script>
    <style>
        {css}
    </style>
</head>
<body>
    <div class="document">
        <header class="title-page">
            <h1 class="book-title">{title}</h1>
            <p class="book-subtitle">Generated with Liminal</p>
        </header>

        <main class="content">
            {content}
        </main>
    </div>

    <div class="watermark">
        <div class="watermark-text">{watermark}</div>
        <a href="{url}" class="watermark-url">{url}</a>
    </div>

    <script>
        hljs.highlightAll();
    </script>
</body>
</html>"##,
        title = html_escape(title),
        content = content,
        watermark = WATERMARK_TEXT,
        url = WEBSITE_URL,
        css = get_pdf_css()
    )
}

fn get_pdf_css() -> &'static str {
    r##"
/* Reset */
*, *::before, *::after {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

/* Page setup for printing */
@page {
    size: A4;
    margin: 2cm 1.5cm 2.5cm 1.5cm;
}

@media print {
    body {
        -webkit-print-color-adjust: exact;
        print-color-adjust: exact;
    }

    .page-break {
        page-break-after: always;
    }

    .title-page {
        page-break-after: always;
    }

    pre {
        page-break-inside: avoid;
    }

    h1, h2, h3, h4 {
        page-break-after: avoid;
    }
}

/* Base styles */
:root {
    --color-bg: #faf8f4;
    --color-bg-elevated: #ffffff;
    --color-text: #2c2416;
    --color-text-secondary: #5a4f3e;
    --color-text-tertiary: #8a7f6e;
    --color-border: rgba(44, 36, 22, 0.12);
    --color-accent: #4a7c59;
    --color-code-bg: #1e1e1e;
    --color-code-text: #d4d4d4;

    --font-serif: 'Libre Baskerville', Georgia, 'Times New Roman', serif;
    --font-mono: 'SF Mono', 'Fira Code', 'Consolas', monospace;
}

html {
    font-size: 11pt;
}

body {
    font-family: var(--font-serif);
    font-size: 1rem;
    line-height: 1.75;
    color: var(--color-text);
    background: var(--color-bg);
}

.document {
    max-width: 100%;
    margin: 0 auto;
    padding: 0 1cm;
}

/* Title page */
.title-page {
    min-height: 90vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    text-align: center;
    padding: 4rem 2rem;
}

.book-title {
    font-family: var(--font-serif);
    font-size: 2.5rem;
    font-weight: 400;
    font-style: italic;
    color: var(--color-text);
    margin-bottom: 1rem;
    line-height: 1.2;
}

.book-subtitle {
    font-family: var(--font-serif);
    font-size: 1rem;
    font-variant: small-caps;
    letter-spacing: 0.15em;
    color: var(--color-text-secondary);
}

/* Content area */
.content {
    padding: 0;
}

.chapter {
    margin-bottom: 2.5rem;
}

/* Typography - Book style */
h1, h2, h3, h4, h5, h6 {
    font-family: var(--font-serif);
    font-weight: 400;
    line-height: 1.3;
    color: var(--color-text);
    margin-top: 1.8em;
    margin-bottom: 0.6em;
}

h1 {
    font-size: 1.8rem;
    font-style: italic;
    border-bottom: 1px solid var(--color-border);
    padding-bottom: 0.4em;
    margin-top: 0;
}

h2 {
    font-size: 1.4rem;
    font-variant: small-caps;
    letter-spacing: 0.05em;
}

h3 {
    font-size: 1.2rem;
    font-style: italic;
}

h4 {
    font-size: 1.05rem;
    font-weight: 600;
}

/* Paragraphs */
p {
    margin: 0 0 0.8em 0;
    text-align: justify;
    hyphens: auto;
}

/* Lists */
ul, ol {
    margin: 1em 0;
    padding-left: 2em;
}

li {
    margin: 0.3em 0;
}

li::marker {
    color: var(--color-text-secondary);
}

/* Blockquotes */
blockquote {
    margin: 1.2em 0;
    padding: 0.8em 1em;
    background: linear-gradient(to right, rgba(74, 124, 89, 0.08), transparent);
    border-left: 3px solid var(--color-accent);
    border-radius: 0 4px 4px 0;
    color: var(--color-text-secondary);
}

blockquote p {
    margin-bottom: 0.4em;
}

blockquote p:last-child {
    margin-bottom: 0;
}

blockquote strong {
    color: var(--color-text);
}

/* Inline code */
code {
    font-family: var(--font-mono);
    font-size: 0.85em;
    background: rgba(44, 36, 22, 0.08);
    padding: 0.15em 0.35em;
    border-radius: 3px;
    color: var(--color-text-secondary);
}

/* Code blocks - use highlight.js styling */
pre {
    background: var(--color-code-bg) !important;
    border-radius: 6px;
    padding: 1em 1.2em;
    overflow-x: auto;
    font-size: 0.8rem;
    line-height: 1.5;
    margin: 1.2em 0;
    text-align: left;
}

pre code {
    background: none !important;
    padding: 0;
    font-size: inherit;
    border-radius: 0;
    color: var(--color-code-text);
}

/* Override highlight.js background */
.hljs {
    background: var(--color-code-bg) !important;
}

/* Horizontal rule */
hr {
    border: none;
    text-align: center;
    margin: 2em 0;
}

hr::before {
    content: '* * *';
    color: var(--color-text-tertiary);
    font-size: 0.9rem;
    letter-spacing: 0.5em;
}

/* Strong and emphasis */
strong {
    font-weight: 700;
}

em {
    font-style: italic;
}

/* Links */
a {
    color: var(--color-text);
    text-decoration: underline;
    text-underline-offset: 2px;
}

/* Watermark - fixed position for every page */
.watermark {
    position: fixed;
    bottom: 0.8cm;
    right: 1cm;
    text-align: right;
    font-family: var(--font-serif);
    font-size: 0.65rem;
    color: var(--color-text-tertiary);
    opacity: 0.7;
}

.watermark-text {
    font-style: italic;
    margin-bottom: 0.15em;
}

.watermark-url {
    font-size: 0.6rem;
    color: var(--color-accent);
    text-decoration: none;
    display: block;
}

.watermark-url:hover {
    text-decoration: underline;
}

/* Page break utility */
.page-break {
    height: 0;
    page-break-after: always;
    margin: 0;
    border: none;
}
"##
}
