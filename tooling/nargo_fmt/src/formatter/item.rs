use noirc_frontend::parser::{Item, ItemKind};

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_item(&mut self, item: Item) {
        self.skip_comments_and_whitespace();

        if !item.doc_comments.is_empty() {
            self.format_outer_doc_comments();
            self.skip_comments_and_whitespace();
        }

        match item.kind {
            ItemKind::Import(_use_tree, _item_visibility) => todo!("Format import"),
            ItemKind::Function(noir_function) => self.format_function(noir_function),
            ItemKind::Struct(noir_struct) => self.format_struct(noir_struct),
            ItemKind::Trait(_noir_trait) => todo!("Format trait"),
            ItemKind::TraitImpl(_noir_trait_impl) => todo!("Format trait impl"),
            ItemKind::Impl(type_impl) => self.format_impl(type_impl),
            ItemKind::TypeAlias(noir_type_alias) => self.format_type_alias(noir_type_alias),
            ItemKind::Global(let_statement, visibility) => {
                self.format_global(let_statement, visibility)
            }
            ItemKind::ModuleDecl(module_declaration) => {
                self.format_module_declaration(module_declaration)
            }
            ItemKind::Submodules(parsed_sub_module) => {
                self.format_submodule(parsed_sub_module);
            }
            ItemKind::InnerAttribute(..) => self.format_inner_attribute(),
        }
    }
}
