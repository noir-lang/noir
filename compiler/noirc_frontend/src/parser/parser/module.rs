use noirc_errors::Span;

use crate::{
    ast::{Ident, ModuleDeclaration},
    parser::{ItemKind, ParsedSubModule},
    token::Attribute,
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_module_or_contract(
        &mut self,
        attributes: Vec<(Attribute, Span)>,
        is_contract: bool,
    ) -> ItemKind {
        let outer_attributes = self.validate_secondary_attributes(attributes);

        if let Some(ident) = self.eat_ident() {
            if self.eat_left_brace() {
                let contents = self.parse_module();
                if !self.eat_right_brace() {
                    // TODO: error
                }
                ItemKind::Submodules(ParsedSubModule {
                    name: ident,
                    contents,
                    outer_attributes,
                    is_contract,
                })
            } else {
                self.eat_semicolons();
                ItemKind::ModuleDecl(ModuleDeclaration { ident, outer_attributes })
            }
        } else {
            // TODO: error
            ItemKind::ModuleDecl(ModuleDeclaration { ident: Ident::default(), outer_attributes })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{parser::parse_program, ItemKind};

    #[test]
    fn parse_module_declaration() {
        // TODO: `contract foo;` is parsed correctly but we don't it's considered a module
        let src = "mod foo;";
        let (module, errors) = parse_program(src);
        assert!(errors.is_empty());
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
        assert!(errors.is_empty());
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
        assert!(errors.is_empty());
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
