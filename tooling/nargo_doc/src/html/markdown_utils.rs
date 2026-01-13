/// Fixes a Markdown string by:
/// - adjusting headers that exceed `current_heading_level` by making them smaller,
///   so headers in user comments aren't larger than the surrounding document's headers.
/// - ensures code blocks are properly closed.
pub(super) fn fix_markdown(markdown: &str, current_heading_level: usize) -> String {
    // Track occurrences of "```" to see if the user forgot to close a code block.
    // If so, we'll close it to prevent ruining the docs.
    let mut open_code_comment = false;
    let mut fixed_comment = String::new();

    'outer_loop: for line in markdown.lines() {
        let trimmed_line = line.trim_start();

        if trimmed_line.starts_with('#') {
            for level in 1..=current_heading_level {
                if trimmed_line.starts_with(&format!("{} ", "#".repeat(level))) {
                    fixed_comment.push_str(&format!(
                        "{} {}",
                        "#".repeat(current_heading_level + 1),
                        &trimmed_line[level + 1..]
                    ));
                    fixed_comment.push('\n');
                    continue 'outer_loop;
                }
            }
        }

        if trimmed_line.starts_with("```") {
            open_code_comment = !open_code_comment;
        }

        fixed_comment.push_str(line);
        fixed_comment.push('\n');
    }

    if open_code_comment {
        fixed_comment.push_str("```");
        fixed_comment.push('\n');
    }

    fixed_comment
}

/// Returns a summary of the given markdown (up to the first blank line).
pub(super) fn markdown_summary(markdown: &str) -> String {
    let mut string = String::new();
    for line in markdown.lines() {
        if line.trim().is_empty() {
            break;
        }
        string.push_str(line);
        string.push('\n');
    }
    let string = string.trim().to_string();
    // Avoid having a header as a summary
    let string =
        if string.starts_with('#') { string.trim_start_matches('#').trim_start() } else { &string };
    string.to_string()
}

pub(super) fn to_html(markdown: &str) -> String {
    let parse = markdown::ParseOptions::default();
    let compile = markdown::CompileOptions {
        // This just means that HTML isn't escaped. Rustdoc works the same way.
        allow_dangerous_html: true,
        ..markdown::CompileOptions::default()
    };
    let options = markdown::Options { parse, compile };
    markdown::to_html_with_options(markdown, &options).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_markdown_adjusts_headers() {
        let input = "# Header 1\n## Header 2\n### Header 3\n#### Header 4\nNormal text";
        let expected = "### Header 1\n### Header 2\n### Header 3\n#### Header 4\nNormal text\n";
        assert_eq!(fix_markdown(input, 2), expected);
    }

    #[test]
    fn test_fix_markdown_closes_code_block_one_block() {
        let input = "Here is some code:\n```\nlet x = 10;\n";
        let expected = "Here is some code:\n```\nlet x = 10;\n```\n";
        assert_eq!(fix_markdown(input, 1), expected);
    }

    #[test]
    fn test_fix_markdown_closes_code_block_two_blocks() {
        let input = "Here is some code:\n```\nlet x = 10;\n```\nMore text.\n```\nlet y = 20;\n";
        let expected =
            "Here is some code:\n```\nlet x = 10;\n```\nMore text.\n```\nlet y = 20;\n```\n";
        assert_eq!(fix_markdown(input, 1), expected);
    }

    #[test]
    fn test_markdown_summary() {
        let input = "# Title\nThis is a summary.\n\nThis part should be ignored.";
        let expected = "Title\nThis is a summary.";
        assert_eq!(markdown_summary(input), expected);
    }
}
