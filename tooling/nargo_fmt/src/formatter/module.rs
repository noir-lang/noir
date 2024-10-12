use noirc_frontend::{
    ast::ModuleDeclaration, parser::ParsedSubModule, token::Keyword, ParsedModule,
};

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_module_declaration(&mut self, module_declaration: ModuleDeclaration) {
        if !module_declaration.outer_attributes.is_empty() {
            self.format_attributes();
        }
        self.write_indentation();
        self.format_item_visibility(module_declaration.visibility);
        self.write_keyword(Keyword::Mod);
        self.write_space();
        self.write_identifier(module_declaration.ident);
        self.write_semicolon();
    }

    pub(super) fn format_submodule(&mut self, submodule: ParsedSubModule) {
        if !submodule.outer_attributes.is_empty() {
            self.format_attributes();
        }
        self.format_item_visibility(submodule.visibility);
        self.write_keyword(Keyword::Mod);
        self.write_space();
        self.write_identifier(submodule.name);
        self.write_space();
        self.write_left_brace();
        if parsed_module_is_empty(&submodule.contents) {
            self.skip_comments_and_whitespace();
        } else {
            self.increase_indentation();
            self.write_line();
            self.format_parsed_module(submodule.contents);
            self.write_line();
            self.deincrease_indentation();
            self.write_indentation();
        }
        self.write_right_brace();
    }
}

fn parsed_module_is_empty(parsed_module: &ParsedModule) -> bool {
    parsed_module.inner_doc_comments.is_empty() && parsed_module.items.is_empty()
}

#[cfg(test)]
mod tests {
    use crate::assert_format;

    #[test]
    fn format_module_declaration() {
        let src = "  mod  foo ; ";
        let expected = "mod foo;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_module_declaration_with_block_comments() {
        let src = "  mod/*a*/ foo /*b*/ ; ";
        let expected = "mod /*a*/ foo /*b*/;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_module_declaration_with_inline_comments() {
        let src = "  mod // a  
 foo // b 
  ; ";
        let expected = "mod // a
foo // b
;
";
        assert_format(src, expected);
    }

    #[test]
    fn format_module_declaration_with_doc_comments() {
        let src = " /// hello
/// world 
mod  foo ; ";
        let expected = "/// hello
/// world
mod foo;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_module_declaration_with_pub_visibility() {
        let src = "  pub   mod  foo  ;";
        let expected = "pub mod foo;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_module_declaration_with_pub_crate_visibility() {
        let src = "  pub ( crate )  mod  foo  ;";
        let expected = "pub(crate) mod foo;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_empty_submodule() {
        let src = "mod foo {    }";
        let expected = "mod foo {}\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_submodule_with_line_comments_in_separate_line() {
        let src = " #[foo] pub  mod foo { 
// one
#[hello]
mod bar; 
// two
}";
        let expected = "#[foo]
pub mod foo {
    // one
    #[hello]
    mod bar;
    // two
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_submodule_with_line_comment_in_same_line() {
        let src = " #[foo] pub  mod foo {  // one
mod bar; 
}";
        let expected = "#[foo]
pub mod foo { // one
    mod bar;
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_submodule_with_block_comment() {
        let src = " #[foo] pub  mod foo {  /* one */
/* two */
mod bar; 
}";
        let expected = "#[foo]
pub mod foo { /* one */
    /* two */
    mod bar;
}
";
        assert_format(src, expected);
    }
}
