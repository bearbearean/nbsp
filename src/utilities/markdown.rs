//! Markdown rendering and HTML sanitizing

use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

use pulldown_cmark::{Options, Parser, html::push_html};

/// The [`ammonia::Builder`]
pub static AMMONIA: LazyLock<ammonia::Builder<'static>> = LazyLock::new(|| {
    let mut sanitizer = ammonia::Builder::new();
    sanitizer.clean_content_tags(HashSet::from_iter(["script", "style"]));
    sanitizer.generic_attributes(HashSet::from_iter([]));
    sanitizer.tag_attributes(HashMap::from_iter([("a", HashSet::from_iter(["href"]))]));
    sanitizer.url_schemes(HashSet::from_iter(["http", "https"]));
    sanitizer.link_rel(Some("noopener noreferrer"));

    #[rustfmt::skip]
    sanitizer.tags(HashSet::from_iter([
        "h1", "h2", "h3", "h4", "h5", "h6",
        "a", "p", "blockquote", "ol", "ul", "li",
        "br", "hr", "strong", "em", "del", "pre", "code",
    ]));

    sanitizer
});

/// Render a Markdown string to HTML and sanitize the resulting HTML
pub fn render_markdown(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(markdown, options);

    let mut unsafe_html = String::new();
    push_html(&mut unsafe_html, parser);

    AMMONIA.clean(&unsafe_html).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_headings() {
        let markdown = "# Title 1";
        let html = render_markdown(markdown);
        assert_eq!(html, "<h1>Title 1</h1>\n");

        let markdown = "Title 1\n===";
        let html = render_markdown(markdown);
        assert_eq!(html, "<h1>Title 1</h1>\n");

        let markdown = "## Title 2";
        let html = render_markdown(markdown);
        assert_eq!(html, "<h2>Title 2</h2>\n");

        let markdown = "Title 2\n---";
        let html = render_markdown(markdown);
        assert_eq!(html, "<h2>Title 2</h2>\n");

        let markdown = "### Title 3";
        let html = render_markdown(markdown);
        assert_eq!(html, "<h3>Title 3</h3>\n");

        let markdown = "#### Title 4";
        let html = render_markdown(markdown);
        assert_eq!(html, "<h4>Title 4</h4>\n");

        let markdown = "##### Title 5";
        let html = render_markdown(markdown);
        assert_eq!(html, "<h5>Title 5</h5>\n");

        let markdown = "###### Title 6";
        let html = render_markdown(markdown);
        assert_eq!(html, "<h6>Title 6</h6>\n");
    }

    #[test]
    fn test_markdown_no_script() {
        let markdown = "<script>window.alert('XSS');</script>";
        let html = render_markdown(markdown);
        assert!(!html.contains("<script>"));
        assert!(!html.contains("</script>"));
        assert!(html.is_empty());
    }

    #[test]
    fn test_markdown_no_style() {
        let markdown = "<style>body {background-color: blue;}</style>";
        let html = render_markdown(markdown);
        assert!(!html.contains("<style>"));
        assert!(!html.contains("</style>"));
        assert!(html.is_empty());
    }

    #[test]
    fn test_markdown_paragraph() {
        let markdown = "This is text";
        let html = render_markdown(markdown);
        assert_eq!(html, "<p>This is text</p>\n");
    }

    #[test]
    fn test_markdown_link() {
        let markdown = "<https://example.com>";
        let html = render_markdown(markdown);
        assert_eq!(
            html,
            "<p><a href=\"https://example.com\" rel=\"noopener noreferrer\">https://example.com</a></p>\n"
        );

        let markdown = "[Link](https://example.com)";
        let html = render_markdown(markdown);
        assert_eq!(
            html,
            "<p><a href=\"https://example.com\" rel=\"noopener noreferrer\">Link</a></p>\n"
        );

        let markdown = "[Link][ref]\n\n[ref]: https://example.com";
        let html = render_markdown(markdown);
        assert_eq!(
            html,
            "<p><a href=\"https://example.com\" rel=\"noopener noreferrer\">Link</a></p>\n"
        );
    }

    #[test]
    fn test_markdown_bold() {
        let markdown = "This is **bold** text";
        let html = render_markdown(markdown);
        assert_eq!(html, "<p>This is <strong>bold</strong> text</p>\n");
    }

    #[test]
    fn test_markdown_italic() {
        let markdown = "This is *italic* text";
        let html = render_markdown(markdown);
        assert_eq!(html, "<p>This is <em>italic</em> text</p>\n");
    }

    #[test]
    fn test_markdown_thematic_break() {
        let markdown = "---";
        let html = render_markdown(markdown);
        assert_eq!(html, "<hr>\n");

        let markdown = "***";
        let html = render_markdown(markdown);
        assert_eq!(html, "<hr>\n");

        let markdown = "___";
        let html = render_markdown(markdown);
        assert_eq!(html, "<hr>\n");
    }

    #[test]
    fn test_markdown_inline_code() {
        let markdown = "This is `inline code`";
        let html = render_markdown(markdown);
        assert_eq!(html, "<p>This is <code>inline code</code></p>\n");
    }

    #[test]
    fn test_markdown_code_blocks() {
        let markdown = "```rust\nprintln!(\"nbsp!\");\n```";
        let html = render_markdown(markdown);
        assert_eq!(html, "<pre><code>println!(\"nbsp!\");\n</code></pre>\n");

        let markdown = "    println!(\"nbsp!\");";
        let html = render_markdown(markdown);
        assert_eq!(html, "<pre><code>println!(\"nbsp!\");</code></pre>\n");
    }

    #[test]
    fn test_markdown_blockquote() {
        let markdown = "> This is a quote";
        let html = render_markdown(markdown);
        assert_eq!(
            html,
            "<blockquote>\n<p>This is a quote</p>\n</blockquote>\n"
        );
    }

    #[test]
    fn test_markdown_ordered_list() {
        let markdown = "1. Item 1\n2. Item 2";
        let html = render_markdown(markdown);
        assert_eq!(html, "<ol>\n<li>Item 1</li>\n<li>Item 2</li>\n</ol>\n");
    }

    #[test]
    fn test_markdown_unordered_list() {
        let markdown = "* Item 1\n* Item 2";
        let html = render_markdown(markdown);
        assert_eq!(html, "<ul>\n<li>Item 1</li>\n<li>Item 2</li>\n</ul>\n");

        let markdown = "- Item 1\n- Item 2";
        let html = render_markdown(markdown);
        assert_eq!(html, "<ul>\n<li>Item 1</li>\n<li>Item 2</li>\n</ul>\n");

        let markdown = "+ Item 1\n+ Item 2";
        let html = render_markdown(markdown);
        assert_eq!(html, "<ul>\n<li>Item 1</li>\n<li>Item 2</li>\n</ul>\n");
    }

    #[test]
    fn test_markdown_line_breaks() {
        let markdown = "Line 1\\\nLine 2  \nLine 3";
        let html = render_markdown(markdown);
        assert_eq!(html, "<p>Line 1<br>\nLine 2<br>\nLine 3</p>\n");
    }

    #[test]
    fn test_markdown_strikethrough() {
        let markdown = "This is ~~strikethrough~~ text";
        let html = render_markdown(markdown);
        assert_eq!(html, "<p>This is <del>strikethrough</del> text</p>\n");
    }
}
