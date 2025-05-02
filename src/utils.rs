// Cargo.toml (add this alongside src/lib.rs):
//
// [package]
// name = "markdown_utils"
// version = "0.1.0"
// edition = "2021"
//
// [dependencies]
// regex = "1"
// base64 = "0.22"
// reqwest = { version = "0.12", features = ["blocking"] }
//
// ------------------------------------------------------------
// src/lib.rs – Markdown utility helpers + equivalent tests
// ------------------------------------------------------------

use base64::{engine::general_purpose, Engine as _};
use regex::Regex;
use std::error::Error;

/// Wrap text with back‑ticks – `inline code`.
pub fn inline_code(text: &str) -> String {
    format!("`{}`", text)
}

/// Wrap an inline equation with single `$` delimiters.
pub fn inline_equation(text: &str) -> String {
    format!("${}$", text)
}

/// Bold – `**text**`.
pub fn bold(text: &str) -> String {
    format!("**{}**", text)
}

/// Italic – `_text_`.
pub fn italic(text: &str) -> String {
    format!("_{}_", text)
}

/// Strikethrough – `~~text~~`.
pub fn strikethrough(text: &str) -> String {
    format!("~~{}~~", text)
}

/// Underline using an inline HTML `<u>` tag (GitHub‑flavoured Markdown passthrough).
pub fn underline(text: &str) -> String {
    format!("<u>{}</u>", text)
}

/// Hyperlink – `[text](href)`.
pub fn link(text: &str, href: &str) -> String {
    format!("[{}]({})", text, href)
}

/// Fenced code‑block with an optional language identifier. Defaults to `plaintext`.
pub fn code_block(text: &str, language: Option<&str>) -> String {
    let lang = language
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .unwrap_or("plaintext")
        .to_lowercase();

    format!("```{}\n{}\n```", lang, text.trim())
}

/// Display equation block (double‑dollar fenced).
pub fn equation(text: &str) -> String {
    format!("$$\n{}\n$$", text.trim())
}

/// Heading helpers.
pub fn heading1(text: &str) -> String {
    format!("# {}", text)
}
pub fn heading2(text: &str) -> String {
    format!("## {}", text)
}
pub fn heading3(text: &str) -> String {
    format!("### {}", text)
}

/// Blockquote – handles multi‑line strings.
pub fn quote(text: &str) -> String {
    format!("> {}", text)
}

/// Representation of a call‑out icon.
#[derive(Debug, Clone)]
pub enum CalloutIcon {
    Emoji(String),
    // In the future additional variants (e.g. built‑in icons) could be added.
}

/// Call‑out block replicating Notion‑style markup.
///
/// Behaviour mirrors the original TS implementation:
/// * Optional emoji prefix
/// * If the first line is a Markdown heading, re‑emit the heading hashes after the quote marker so
///   that the heading level is preserved inside the call‑out.
pub fn callout(text: &str, icon: Option<CalloutIcon>) -> String {
    let emoji_prefix = match icon {
        Some(CalloutIcon::Emoji(e)) => format!("{} ", e),
        None => String::new(),
    };

    let formatted_text = text.replace('\n', "  \n> ");
    let re = Regex::new(r"^(#{1,6})\s+([\s\S]+)").unwrap();

    if let Some(caps) = re.captures(&formatted_text) {
        // Preserve heading level inside the call‑out.
        let hashes = &caps[1];
        let content = &caps[2];
        return format!("> {}{} {}", hashes, if emoji_prefix.is_empty() { "" } else { " " }, emoji_prefix.trim_end()).trim_end().to_owned() + content;
    }

    format!("> {}{}", emoji_prefix, formatted_text)
}

/// Unordered / ordered list helpers.
pub fn bullet(text: &str, count: Option<usize>) -> String {
    let trimmed = text.trim();
    match count {
        Some(n) => format!("{}. {}", n, trimmed),
        None => format!("- {}", trimmed),
    }
}

/// Task‑list item.
pub fn todo(text: &str, checked: bool) -> String {
    if checked {
        format!("- [x] {}", text)
    } else {
        format!("- [ ] {}", text)
    }
}

/// Helper for inserting tab‑level indentation (4‑space "tabs").
pub fn add_tab_space(text: &str, n: usize) -> String {
    let tab = "    ";
    let mut out = String::from(text);
    for _ in 0..n {
        if out.contains('\n') {
            out = out.split('\n').collect::<Vec<_>>().join(&format!("\n{}", tab));
            out = format!("{}{}", tab, out);
        } else {
            out = format!("{}{}", tab, out);
        }
    }
    out
}

/// Horizontal rule.
pub fn divider() -> &'static str {
    "---"
}

/// Details/summary toggle block.
pub fn toggle(summary: Option<&str>, children: Option<&str>) -> String {
    match (summary, children) {
        (None, None) => "".into(),
        (None, Some(c)) => c.into(),
        (Some(s), content) => format!(
            "<details>\n<summary>{}</summary>\n{}\n</details>\n\n",
            s,
            content.unwrap_or("")
        ),
    }
}

/// Simple Markdown table generator.
/// Pads each column to the width of the longest cell – sufficient for unit‑test purposes.
pub fn table(rows: &[Vec<&str>]) -> String {
    assert!(!rows.is_empty(), "table requires at least one row");

    let cols = rows[0].len();
    let mut col_widths = vec![0usize; cols];
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            col_widths[i] = col_widths[i].max(cell.len());
        }
    }

    let fmt_row = |row: &[&str]| -> String {
        let formatted: Vec<String> = row
            .iter()
            .enumerate()
            .map(|(i, cell)| {
                let pad = col_widths[i] - cell.len();
                format!(" {}{} ", cell, " ".repeat(pad))
            })
            .collect();
        format!("|{}|", formatted.join("|"))
    };

    let header = fmt_row(&rows[0]);
    let separator = {
        let parts: Vec<String> = col_widths
            .iter()
            .map(|w| format!(" {} ", "-".repeat(*w)))
            .collect();
        format!("|{}|", parts.join("|"))
    };

    let mut out = vec![header, separator];
    for row in &rows[1..] {
        out.push(fmt_row(row));
    }

    out.join("\n")
}

/// Helper that converts an image URL to Markdown, optionally embedding as base64.
/// Follows the behaviour of the original JS implementation.
/// * If `convert_to_base64` is false, or the href already contains a `data:` URI, we simply emit it.
/// * Otherwise we synchronously download the image and embed the base64 payload (PNG‑assumed).
pub fn image(
    alt: &str,
    href: &str,
    convert_to_base64: bool,
) -> Result<String, Box<dyn Error>> {
    if !convert_to_base64 || href.starts_with("data:") {
        if href.starts_with("data:") {
            // Attempt to normalise to PNG MIME
            let base64_data = href.splitn(2, ',').nth(1).unwrap_or("");
            return Ok(format!(
                "![{}](data:image/png;base64,{})",
                alt, base64_data
            ));
        }
        return Ok(format!("![{}]({})", alt, href));
    }

    // Blocking download
    let bytes = reqwest::blocking::get(href)?.bytes()?;
    let encoded = general_purpose::STANDARD.encode(bytes);
    Ok(format!(
        "![{}](data:image/png;base64,{})",
        alt, encoded
    ))
}

// ------------------------------------------------------------
//                               Tests
// ------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn nospace(s: &str) -> String {
        s.chars().filter(|c| !c.is_whitespace()).collect()
    }

    // ---------------- Callout ----------------
    #[test]
    fn callout_without_emoji() {
        let text = "Call out text content.";
        assert_eq!(callout(text, None), format!("> {}", text));
    }

    #[test]
    fn callout_with_emoji() {
        let text = "Call out text content.";
        assert_eq!(
            callout(text, Some(CalloutIcon::Emoji("😍".into()))),
            format!("> 😍 {}", text)
        );
    }

    // --------------- Markdown Table ----------
    #[test]
    fn simple_table() {
        let mock = vec![
            vec!["number", "char"],
            vec!["1", "a"],
            vec!["2", "b"],
        ];
        let expected = "| number | char |\n| ------ | ---- |\n| 1      | a    |\n| 2      | b    |";
        assert_eq!(table(&mock), expected);
    }

    // --------------- Text Annotations --------
    #[test]
    fn inline_code_test() {
        assert_eq!(inline_code("simple text"), "`simple text`");
    }

    #[test]
    fn code_block_test() {
        let expected = "```javascript\nsimple text\n```";
        assert_eq!(code_block("simple text", Some("javascript")), expected);
    }

    #[test]
    fn inline_equation_test() {
        assert_eq!(inline_equation("E = mc^2"), "$E = mc^2$");
    }

    #[test]
    fn equation_block_test() {
        let expected = "$$\nE = mc^2\n$$";
        assert_eq!(equation("E = mc^2"), expected);
    }

    #[test]
    fn bold_test() {
        assert_eq!(bold("simple text"), "**simple text**");
    }

    #[test]
    fn italic_test() {
        assert_eq!(italic("simple text"), "_simple text_");
    }

    #[test]
    fn strikethrough_test() {
        assert_eq!(strikethrough("simple text"), "~~simple text~~");
    }

    #[test]
    fn underline_test() {
        assert_eq!(underline("simple text"), "<u>simple text</u>");
    }

    // ---------------- Headings ---------------
    #[test]
    fn heading1_test() {
        assert_eq!(heading1("simple text"), "# simple text");
    }
    #[test]
    fn heading2_test() {
        assert_eq!(heading2("simple text"), "## simple text");
    }
    #[test]
    fn heading3_test() {
        assert_eq!(heading3("simple text"), "### simple text");
    }

    // ---------------- List Elements ----------
    #[test]
    fn bullet_test() {
        assert_eq!(bullet("simple text", None), "- simple text");
    }

    #[test]
    fn checked_todo_test() {
        assert_eq!(todo("simple text", true), "- [x] simple text");
    }

    #[test]
    fn unchecked_todo_test() {
        assert_eq!(todo("simple text", false), "- [ ] simple text");
    }

    // ---------------- Image ------------------
    #[test]
    fn image_with_alt_text() {
        let out = image("simple text", "https://example.com/image", false).unwrap();
        assert_eq!(out, "![simple text](https://example.com/image)");
    }

    #[test]
    #[ignore] // Network call – run with `cargo test -- --ignored` to include.
    fn image_to_base64() {
        let md = image(
            "simple text",
            "https://w.wallhaven.cc/full/ex/wallhaven-ex9gwo.png",
            true,
        )
        .unwrap();
        assert!(md.starts_with(
            "![simple text](data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAB4AAAAQ4CAYAAADo08FDAAAgAElEQVR4Aby9O5OlW5r"
        ));
    }

    // ---------------- Toggle -----------------
    #[test]
    fn toggle_no_summary() {
        assert_eq!(nospace(&toggle(None, Some("content"))), "content");
    }

    #[test]
    fn toggle_empty() {
        assert_eq!(nospace(&toggle(None, None)), "");
    }

    #[test]
    fn toggle_details_summary() {
        assert_eq!(
            nospace(&toggle(Some("title"), Some("content"))),
            "<details><summary>title</summary>content</details>"
        );
    }
}
