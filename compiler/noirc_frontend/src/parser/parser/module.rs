use noirc_errors::Span;

use crate::{
    ast::{Ident, ItemVisibility, ModuleDeclaration},
    parser::{ItemKind, ParsedSubModule},
    token::{Attribute, Token},
};

use super::Parser;

impl<'a> Parser<'a> {
    /// ModOrContract
    ///     = ( 'mod' | 'contract' ) identifier ( '{' Module '}' | ';' )
    pub(super) fn parse_mod_or_contract(
        &mut self,
        attributes: Vec<(Attribute, Span)>,
        is_contract: bool,
        visibility: ItemVisibility,
    ) -> ItemKind {
        let outer_attributes = self.validate_secondary_attributes(attributes);

        let Some(ident) = self.eat_ident() else {
            self.expected_identifier();
            return ItemKind::ModuleDecl(ModuleDeclaration {
                visibility,
                ident: Ident::default(),
                outer_attributes,
            });
        };

        if self.eat_left_brace() {
            let contents = self.parse_module(
                true, // nested
            );
            self.eat_or_error(Token::RightBrace);
            ItemKind::Submodules(ParsedSubModule {
                visibility,
                name: ident,
                contents,
                outer_attributes,
                is_contract,
            })
        } else {
            if !self.eat_semicolons() {
                self.expected_token(Token::Semicolon);
            }
            ItemKind::ModuleDecl(ModuleDeclaration { visibility, ident, outer_attributes })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{
        parser::{parse_program, tests::expect_no_errors},
        ItemKind,
    };

    #[test]
    fn parse_module_declaration() {
        // TODO: `contract foo;` is parsed correctly but we don't it's considered a module
        let src = "mod foo;";
        let (module, errors) = parse_program(src);
        expect_no_errors(&errors);
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::ModuleDecl(module) = &item.kind else {
            panic!("Expected module declaration");
        };
        assert_eq!("foo", module.ident.to_string());
    }

    #[test]
    fn parse_submodule() {
        let src = "mod foo { mod bar; }";
        let (module, errors) = parse_program(src);
        expect_no_errors(&errors);
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Submodules(parsed_submodule) = &item.kind else {
            panic!("Expected submodules declaration");
        };
        assert!(!parsed_submodule.is_contract);
        assert_eq!("foo", parsed_submodule.name.to_string());
        assert_eq!(parsed_submodule.contents.items.len(), 1);
    }

    #[test]
    fn parse_contract() {
        let src = "contract foo {}";
        let (module, errors) = parse_program(src);
        expect_no_errors(&errors);
        assert_eq!(module.items.len(), 1);
        let item = &module.items[0];
        let ItemKind::Submodules(parsed_submodule) = &item.kind else {
            panic!("Expected submodules declaration");
        };
        assert!(parsed_submodule.is_contract);
        assert_eq!("foo", parsed_submodule.name.to_string());
        assert_eq!(parsed_submodule.contents.items.len(), 0);
    }
}
