use noirc_frontend::parser::{Item, ItemKind};

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_item(&mut self, item: Item) {
        if !item.doc_comments.is_empty() {
            self.format_outer_doc_comments();
        }

        self.skip_comments_and_whitespace();
        match item.kind {
            ItemKind::Import(_use_tree, _item_visibility) => todo!(),
            ItemKind::Function(noir_function) => self.format_function(noir_function),
            ItemKind::Struct(noir_struct) => self.format_struct(noir_struct),
            ItemKind::Trait(_noir_trait) => todo!(),
            ItemKind::TraitImpl(_noir_trait_impl) => todo!(),
            ItemKind::Impl(_type_impl) => todo!(),
            ItemKind::TypeAlias(_noir_type_alias) => todo!(),
            ItemKind::Global(_let_statement, _item_visibility) => todo!(),
            ItemKind::ModuleDecl(module_declaration) => {
                self.format_module_declaration(module_declaration)
            }
            ItemKind::Submodules(parsed_sub_module) => {
                self.format_submodule(parsed_sub_module);
            }
            ItemKind::InnerAttribute(..) => {
                self.format_inner_doc_comment();
            }
        }
    }
}
