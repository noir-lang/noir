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
        self.write_line();
    }

    pub(super) fn format_submodule(&mut self, submodule: ParsedSubModule) {
        if !submodule.outer_attributes.is_empty() {
            self.format_attributes();
        }
        self.write_indentation();
        self.format_item_visibility(submodule.visibility);
        if submodule.is_contract {
            self.write_keyword(Keyword::Contract);
        } else {
            self.write_keyword(Keyword::Mod);
        }
        self.write_space();
        self.write_identifier(submodule.name);
        self.write_space();
        if parsed_module_is_empty(&submodule.contents) {
            self.write_left_brace();
            self.increase_indentation();
            let skip_result = self.skip_comments_and_whitespace_writing_lines_if_found();
            self.decrease_indentation();
            if skip_result.wrote_comment {
                self.write_line();
                self.write_indentation();
            }
            self.write_right_brace();
        } else {
            self.write_left_brace();
            self.increase_indentation();
            self.write_line();
            self.format_parsed_module(submodule.contents);
            self.decrease_indentation();
            self.write_indentation();
            self.write_right_brace();
        }
        self.write_line();
    }
}

fn parsed_module_is_empty(parsed_module: &ParsedModule) -> bool {
    parsed_module.inner_doc_comments.is_empty() && parsed_module.items.is_empty()
}

#[cfg(test)]
mod tests {
    use crate::{assert_format, assert_format_with_config, Config};

    #[test]
    fn format_module_declaration() {
        let src = "  mod  foo ; ";
        let expected = "mod foo;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_module_declaration_with_block_comments() {
        let src = "  mod/*a*/ foo /*b*/ ; ";
        let expected = "mod/*a*/ foo /*b*/;\n";
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
    fn format_empty_subcontract() {
        let src = "contract foo {    }";
        let expected = "contract foo {}\n";
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

    #[test]
    fn format_submodule_with_block_comment_2() {
        let src = "mod foo {
        /* one */
}";
        let expected = "mod foo {
    /* one */
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_multiple_modules() {
        let src = "  mod  foo { 
// hello
mod bar {
// world
}
} ";
        let expected = "mod foo {
    // hello
    mod bar {
        // world
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn keeps_spaces_between_comments() {
        let src = "  mod  foo { 

// hello

// world

} ";
        let expected = "mod foo {

    // hello

    // world

}
";
        assert_format(src, expected);
    }

    #[test]
    fn comment_with_leading_space() {
        let src = "    // comment
        // hello
mod  foo ; ";
        let expected = "// comment
// hello
mod foo;
";
        assert_format(src, expected);
    }

    #[test]
    fn format_submodule_with_configurable_tab_spaces() {
        let src = " mod foo {
        mod bar ;
    }";
        let expected = "mod foo {
  mod bar;
}
";
        let config = Config { tab_spaces: 2, ..Config::default() };
        assert_format_with_config(src, expected, config);
    }
}
