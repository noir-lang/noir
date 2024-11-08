use noirc_frontend::{
    ast::ModuleDeclaration, parser::ParsedSubModule, token::Keyword, ParsedModule,
};

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_module_declaration(&mut self, module_declaration: ModuleDeclaration) {
        self.format_secondary_attributes(module_declaration.outer_attributes);
        self.write_indentation();
        self.format_item_visibility(module_declaration.visibility);
        self.write_keyword(Keyword::Mod);
        self.write_space();
        self.write_identifier(module_declaration.ident);
        self.write_semicolon();
    }

    pub(super) fn format_submodule(&mut self, submodule: ParsedSubModule) {
        self.format_secondary_attributes(submodule.outer_attributes);
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
        self.write_left_brace();
        if parsed_module_is_empty(&submodule.contents) {
            self.format_empty_block_contents();
        } else {
            self.increase_indentation();
            self.write_line();
            self.format_parsed_module(submodule.contents, self.ignore_next);
            self.decrease_indentation();
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
    use crate::{assert_format, assert_format_with_config, Config};

    #[test]
    fn format_module_declaration() {
        let src = "  mod  foo ; ";
        let expected = "mod foo;\n";
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
    fn format_empty_submodule_2() {
        let src = "mod foo { mod bar {    

    } }";
        let expected = "mod foo {
    mod bar {}
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_empty_subcontract() {
        let src = "contract foo {    }";
        let expected = "contract foo {}\n";
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
