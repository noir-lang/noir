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
    }

    fixed_comment.push('\n');
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
    let string = string.trim_start_matches('#');
    let markdown = markdown::to_html(string);
    let markdown = markdown.trim_start_matches("<p>");
    markdown.trim_end_matches("</p>").trim().to_string()
}
