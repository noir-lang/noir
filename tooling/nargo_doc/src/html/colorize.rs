use noirc_frontend::{
    lexer::Lexer,
    token::{FmtStrFragment, LocatedToken, Token},
};

use crate::html::escape_html;

pub(super) fn colorize_code_blocks(comments: String) -> String {
    if !comments.contains("```") {
        return comments;
    }

    let mut in_code_block = false; // Are we inside a code block?
    let mut highlighting = false; // Do we need to highlight code?
    let mut result = String::new();

    let lines = comments.lines().collect::<Vec<_>>();
    for (index, line) in lines.iter().enumerate() {
        let trimmed_line = line.trim();
        if trimmed_line.starts_with("```") {
            in_code_block = !in_code_block;
        }

        // We assume code blocks without a name are for Noir.
        // We also colorize Rust because the two langauges are very similar, and because some
        // of our stdlib docs use Rust code blocks.
        if in_code_block
            && (trimmed_line == "```" || trimmed_line == "```noir" || trimmed_line == "```rust")
        {
            result.push_str("<pre><code>");
            highlighting = true;
        } else if !in_code_block && trimmed_line == "```" {
            if highlighting {
                result.push_str("</pre></code>");
            } else {
                result.push_str("```");
            }
            highlighting = false;
        } else {
            if highlighting {
                result.push_str(&colorize_code_line(line));
            } else {
                result.push_str(line);
            }
            if index != lines.len() - 1 {
                result.push('\n');
            }
        }
    }

    if in_code_block && highlighting {
        result.push_str("</pre></code>");
    }

    result
}

fn colorize_code_line(line: &str) -> String {
    let mut result = String::new();
    let lexer = Lexer::new_with_dummy_file(line).skip_comments(false).skip_whitespaces(false);

    for token in lexer {
        let Ok(token) = token else {
            // If lexing fails, give up and return the original line
            return line.to_string();
        };
        if matches!(token.token(), Token::EOF) {
            break;
        }

        colorize_token(&token, line, &mut result);
    }

    result
}

fn colorize_token(token: &LocatedToken, line: &str, result: &mut String) {
    let span = token.span();
    let token = token.token();
    let token_str = &line[span.start() as usize..span.end() as usize];
    let class = match token {
        Token::Int(..) => Some("number"),
        Token::Bool(_) => Some("bool-var"),
        Token::Str(_) | Token::RawStr(_, _) => Some("string"),
        Token::FmtStr(fragments, _) => {
            result.push_str("<span class=\"string\">f\"</span>");
            for fragment in fragments {
                match fragment {
                    FmtStrFragment::String(string) => {
                        result.push_str(&format!(
                            "<span class=\"string\">{}</span>",
                            escape_html(string)
                        ));
                    }
                    FmtStrFragment::Interpolation(string, _) => {
                        result.push_str("<span class=\"interpolation\">{");
                        result.push_str(string);
                        result.push_str("}</span>");
                    }
                }
            }
            result.push_str("<span class=\"string\">\"</span>");
            return;
        }
        Token::Keyword(_) => Some("kw"),
        Token::AttributeStart { .. } => None,
        Token::LineComment(_, doc_style) | Token::BlockComment(_, doc_style) => match doc_style {
            Some(..) => Some("doccomment"),
            None => Some("comment"),
        },
        Token::Quote(tokens) => {
            result.push_str("<span class=\"kw\">quote</span> {");
            for token in &tokens.0 {
                colorize_token(token, line, result);
            }
            result.push('}');
            return;
        }
        Token::QuotedType(_)
        | Token::Ident(_)
        | Token::InternedExpr(..)
        | Token::InternedStatement(..)
        | Token::InternedLValue(..)
        | Token::InternedUnresolvedTypeData(..)
        | Token::InternedPattern(..)
        | Token::InternedCrate(..)
        | Token::Less
        | Token::LessEqual
        | Token::Greater
        | Token::GreaterEqual
        | Token::Equal
        | Token::NotEqual
        | Token::Plus
        | Token::Minus
        | Token::Star
        | Token::Slash
        | Token::Percent
        | Token::Ampersand
        | Token::SliceStart
        | Token::Caret
        | Token::ShiftLeft
        | Token::ShiftRight
        | Token::Dot
        | Token::DoubleDot
        | Token::DoubleDotEqual
        | Token::LeftParen
        | Token::RightParen
        | Token::LeftBrace
        | Token::RightBrace
        | Token::LeftBracket
        | Token::RightBracket
        | Token::Arrow
        | Token::FatArrow
        | Token::Pipe
        | Token::Pound
        | Token::Comma
        | Token::Colon
        | Token::DoubleColon
        | Token::Semicolon
        | Token::Bang
        | Token::Assign
        | Token::DollarSign
        | Token::LogicalAnd
        | Token::EOF
        | Token::Whitespace(_)
        | Token::UnquoteMarker(_)
        | Token::Invalid(_) => None,
    };
    if let Some(class) = class {
        result.push_str(&format!("<span class=\"{class}\">{}</span>", escape_html(token_str)));
    } else {
        result.push_str(&escape_html(token_str));
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::html::colorize_code_blocks;

    #[test]
    fn test_colorize_code_blocks() {
        let markdown = r#"
Colorized:

```
let bool_var = true;
let number = 1;
let string = "hello";
let f = f"Interpolated {string} and {number}";
let quoted = quote { 1 + 2 };

// Line comment
/// Doc comment
#[attribute]
fn foo() {}
```

Also colorized:

```noir
let bool_var = true;
```

Also colorized:

```rust
let bool_var = true;
```

Not colorized:

```text
let bool_var = true;
```

        "#;
        let colorized = colorize_code_blocks(markdown.to_string());
        assert_snapshot!(colorized, @r#"
        Colorized:

        <pre><code><span class="kw">let</span> bool_var = <span class="bool-var">true</span>;
        <span class="kw">let</span> number = <span class="number">1</span>;
        <span class="kw">let</span> string = <span class="string">"hello"</span>;
        <span class="kw">let</span> f = <span class="string">f"</span><span class="string">Interpolated </span><span class="interpolation">{string}</span><span class="string"> and </span><span class="interpolation">{number}</span><span class="string">"</span>;
        <span class="kw">let</span> quoted = <span class="kw">quote</span> { <span class="number">1</span> + <span class="number">2</span> };

        <span class="comment">// Line comment</span>
        <span class="doccomment">/// Doc comment</span>
        #[attribute]
        <span class="kw">fn</span> foo() {}
        </pre></code>
        Also colorized:

        <pre><code><span class="kw">let</span> bool_var = <span class="bool-var">true</span>;
        </pre></code>
        Also colorized:

        <pre><code><span class="kw">let</span> bool_var = <span class="bool-var">true</span>;
        </pre></code>
        Not colorized:

        ```text
        let bool_var = true;
        ```
        "#);
    }
}
