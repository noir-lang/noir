use noirc_frontend::ast::Documented;
use noirc_frontend::ast::LetStatement;
use noirc_frontend::ast::ModuleDeclaration;
use noirc_frontend::ast::NoirFunction;
use noirc_frontend::ast::NoirStruct;
use noirc_frontend::ast::NoirTrait;
use noirc_frontend::ast::NoirTypeAlias;
use noirc_frontend::parser::SortedSubModule;

fn short_description(doc_comments: &[String]) -> String {
    doc_comments
        .iter()
        .take_while(|line| !line.is_empty())
        .fold(String::new(), |acc, line| format!("{acc} {line}"))
}

pub struct Metadata {
    pub title: String,
    pub short_description: String,
}

impl From<&Documented<NoirFunction>> for Metadata {
    fn from(value: &Documented<NoirFunction>) -> Self {
        Self {
            title: value.item.name().to_string(),
            short_description: short_description(&value.doc_comments),
        }
    }
}

impl From<&Documented<NoirStruct>> for Metadata {
    fn from(value: &Documented<NoirStruct>) -> Self {
        Self {
            title: value.item.name.to_string(),
            short_description: short_description(&value.doc_comments),
        }
    }
}

impl From<&Documented<NoirTrait>> for Metadata {
    fn from(value: &Documented<NoirTrait>) -> Self {
        Self {
            title: value.item.name.to_string(),
            short_description: short_description(&value.doc_comments),
        }
    }
}

impl From<&Documented<NoirTypeAlias>> for Metadata {
    fn from(value: &Documented<NoirTypeAlias>) -> Self {
        Self {
            title: value.item.name.to_string(),
            short_description: short_description(&value.doc_comments),
        }
    }
}

impl From<&Documented<LetStatement>> for Metadata {
    fn from(value: &Documented<LetStatement>) -> Self {
        Self {
            title: value.item.pattern.name_ident().0.contents.clone(),
            short_description: short_description(&value.doc_comments),
        }
    }
}

impl From<&Documented<ModuleDeclaration>> for Metadata {
    fn from(value: &Documented<ModuleDeclaration>) -> Self {
        Self {
            title: value.item.ident.to_string(),
            short_description: short_description(&value.doc_comments),
        }
    }
}

impl From<&Documented<SortedSubModule>> for Metadata {
    fn from(value: &Documented<SortedSubModule>) -> Self {
        Self {
            title: value.item.name.to_string(),
            short_description: short_description(&value.doc_comments),
        }
    }
}
