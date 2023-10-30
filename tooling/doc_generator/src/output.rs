use std::fmt;

use noirc_frontend::token::{DocComments, Keyword, SpannedToken, Token};

use crate::{
    additional_doc, doc, fn_signature, get_module_content, outer_doc, skip_impl_block,
    struct_signature, trait_info, Function, Implementation,
};

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) enum Type {
    Function,
    Module,
    Struct,
    Trait,
    OuterComment,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Function => write!(f, "Function"),
            Type::Module => write!(f, "Module"),
            Type::Struct => write!(f, "Struct"),
            Type::Trait => write!(f, "Trait"),
            Type::OuterComment => write!(f, "OuterComment"),
        }
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub(crate) enum Info {
    Function {
        signature: String,
    },
    Module {
        content: Vec<Output>,
    },
    Struct {
        signature: String,
        additional_doc: String,
        implementations: Vec<Implementation>,
    },
    Trait {
        signature: String,
        additional_doc: String,
        required_methods: Vec<Function>,
        provided_methods: Vec<Function>,
        implementations: Vec<Implementation>,
    },
    Blanc,
}

impl Info {
    pub(crate) fn get_signature(&self) -> Option<String> {
        match self {
            Info::Function { signature } => Some(signature.to_string()),
            Info::Struct { signature, .. } => Some(signature.to_string()),
            Info::Trait { signature, .. } => Some(signature.to_string()),
            _ => None,
        }
    }

    pub(crate) fn get_implementations(&self) -> Option<Vec<Implementation>> {
        match self {
            Info::Struct { implementations, .. } => Some(implementations.clone()),
            Info::Trait { implementations, .. } => Some(implementations.clone()),
            _ => None,
        }
    }

    pub(crate) fn get_additional_doc(&self) -> Option<String> {
        match self {
            Info::Struct { additional_doc, .. } => Some(additional_doc.to_string()),
            Info::Trait { additional_doc, .. } => Some(additional_doc.to_string()),
            _ => None,
        }
    }

    pub(crate) fn get_required_methods(&self) -> Option<Vec<Function>> {
        match self {
            Info::Trait { required_methods, .. } => Some(required_methods.clone()),
            _ => None,
        }
    }

    pub(crate) fn get_provided_methods(&self) -> Option<Vec<Function>> {
        match self {
            Info::Trait { provided_methods, .. } => Some(provided_methods.clone()),
            _ => None,
        }
    }

    pub(crate) fn get_content(&self) -> Option<Vec<Output>> {
        match self {
            Info::Module { content } => Some(content.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct Output {
    pub(crate) r#type: Type,
    pub(crate) name: String,
    pub(crate) doc: String,
    pub(crate) information: Info,
}

impl Output {
    pub(crate) fn to_output(input: Vec<SpannedToken>) -> Vec<Self> {
        let mut res = Vec::new();
        let tokens = input.into_iter().map(|x| x.into_token()).collect::<Vec<_>>();
        let mut is_first = true;
        let mut skip_count = 0;

        for i in 0..tokens.len() {
            if skip_count > 0 {
                skip_count -= 1;
                continue;
            }
            let out = match &tokens[i] {
                Token::Keyword(Keyword::Fn) => {
                    let r#type = Type::Function;
                    let name = match &tokens[i + 1] {
                        Token::Ident(idn) => idn.clone(),
                        _ => {
                            continue;
                        }
                    };
                    let doc = doc(&tokens, i);
                    let sign = fn_signature(&tokens, i);

                    Output { r#type, name, doc, information: Info::Function { signature: sign } }
                }
                Token::Keyword(Keyword::Struct) => {
                    let r#type = Type::Struct;
                    let name = match &tokens[i + 1] {
                        Token::Ident(idn) => idn.clone(),
                        _ => {
                            continue;
                        }
                    };
                    let doc = doc(&tokens, i);
                    let sign = struct_signature(&tokens, i - 1);
                    let ad_doc = additional_doc(&tokens, i);

                    Output {
                        r#type,
                        name: name.clone(),
                        doc,
                        information: Info::Struct {
                            signature: sign,
                            additional_doc: ad_doc,
                            implementations: Implementation::get_implementations(&tokens, i, name),
                        },
                    }
                }
                Token::Keyword(Keyword::Trait) => {
                    skip_count = skip_impl_block(&tokens, i);

                    let r#type = Type::Trait;
                    let name = match &tokens[i + 1] {
                        Token::Ident(idn) => idn.clone(),
                        _ => {
                            continue;
                        }
                    };
                    let doc = doc(&tokens, i);

                    let ad_doc = additional_doc(&tokens, i);
                    let impls = Implementation::get_implementations(&tokens, i, name.clone());
                    let info = trait_info(&tokens, i);

                    Output {
                        r#type,
                        name,
                        doc,
                        information: Info::Trait {
                            signature: info.0,
                            additional_doc: ad_doc,
                            required_methods: info.1,
                            provided_methods: info.2,
                            implementations: impls,
                        },
                    }
                }
                Token::Keyword(Keyword::Mod) => {
                    if tokens[i + 2] == Token::LeftBrace {
                        skip_count = skip_impl_block(&tokens, i);
                    }

                    let r#type = Type::Module;
                    let name = match &tokens[i + 1] {
                        Token::Ident(idn) => idn.clone(),
                        _ => {
                            continue;
                        }
                    };
                    let doc = doc(&tokens, i);
                    let content = get_module_content(&tokens, i);

                    Output { r#type, name, doc, information: Info::Module { content } }
                }
                Token::DocComment(DocComments::Outer(_)) => {
                    let r#type = Type::OuterComment;
                    let name = "".to_string();

                    let res = outer_doc(&tokens, i);

                    let doc = if is_first {
                        is_first = false;
                        res.0
                    } else {
                        if res.1 == i {
                            is_first = true;
                        }
                        "".to_string()
                    };

                    Output { r#type, name, doc, information: Info::Blanc }
                }
                Token::Keyword(Keyword::Impl) => {
                    skip_count = skip_impl_block(&tokens, i);
                    continue;
                }
                _ => {
                    continue;
                }
            };

            res.push(out);
        }

        res
    }
}

impl fmt::Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Type: {:?}\n", self.r#type)?;
        write!(f, "Name: {}\n", self.name)?;
        write!(f, "Doc: {}\n", self.doc)?;
        Ok(())
    }
}
