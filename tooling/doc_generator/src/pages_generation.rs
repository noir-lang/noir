use std::{
    ffi::OsStr,
    fs::{self, File},
    io::Write,
    path::Path,
};

use askama::Template;
use noirc_frontend::token::{Keyword, Token};

use crate::{doc, fn_signature, get_text, Code, Output, Type};

/// Generates an HTML code page from the content of a text file.

/// The `generate_code_page` function reads the content of a text file specified by the `input_file` path,
/// processes the text content as code lines, and generates an HTML page that displays the code lines.
/// The resulting HTML page can be used for code documentation or rendering purposes.
fn generate_code_page(input_file: &str) -> Result<(), crate::DocError> {
    let codelines = get_text(input_file)?;

    let code = Code { codelines };

    let rendered_html = code.render().map_err(|_| crate::DocError::RenderError)?;

    let fname = format!(
        "generated_doc/codepage_{}.html",
        extract_filename(input_file).ok_or(crate::DocError::ExtractFilenameError)?
    );

    let mut file = File::create(fname).map_err(|_| crate::DocError::FileCreateError)?;
    file.write_all(rendered_html.as_bytes()).map_err(|_| crate::DocError::FileEditError)?;

    Ok(())
}

/// Represents a function with associated documentation and signature.

/// The `Function` struct is used to represent a function, providing information about its name,
/// documentation, signature, and whether it's a method. This structure is typically used to organize
/// and process information related to functions within the code.
#[derive(Debug, Clone, Template, Eq, Hash, PartialEq)]
#[template(path = "func_template.html")]
pub(crate) struct Function {
    pub(crate) name: String,
    pub(crate) doc: String,
    pub(crate) signature: String,
    pub(crate) is_method: bool,
}

/// Generates an HTML page for a function with associated documentation.

/// The `generate_function_pages` function generates an HTML page for a function represented by the `func` parameter.
/// The HTML page displays the function's name, documentation, and signature. This function is typically used to
/// generate documentation pages for functions.
fn generate_function_pages(func: Function) -> Result<(), crate::DocError> {
    if func.is_method {
        return Ok(());
    }
    let rendered_html = func.render().map_err(|_| crate::DocError::RenderError)?;

    let output_file_name = format!("generated_doc/{}.html", func.name);

    let mut file = File::create(output_file_name).map_err(|_| crate::DocError::FileCreateError)?;
    file.write_all(rendered_html.as_bytes()).map_err(|_| crate::DocError::FileEditError)?;

    Ok(())
}

/// Represents a structured code element, typically a struct, with associated documentation and details.

/// The `Structure` struct is used to represent a structured code element, such as a struct. It provides information
/// about the name, documentation, additional documentation, signature, and implementations related to the struct.
/// This structure is designed to organize and process information related to structured code elements.
#[derive(Debug, Template)]
#[template(path = "struct_template.html")]
pub(crate) struct Structure {
    name: String,
    doc: String,
    additional_doc: String,
    signature: String,
    implementations: Vec<Implementation>,
}

/// Represents an implementation of methods for a code element.

/// The `Implementation` struct provides information about the signature of an implementation and the associated
/// functions within the implementation. It is used to organize and process information related to the implementation
/// of methods for a code element.
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub(crate) struct Implementation {
    signature: String,
    functions: Vec<Function>,
}

impl Implementation {
    /// Retrieves a list of implementations for a code element.

    /// The `get_implementations` method is used to retrieve a list of implementations for a code element.
    /// It searches through the provided tokens and identifies implementations associated with the original code element.
    pub(crate) fn get_implementations(
        tokens: &[Token],
        index: usize,
        orig_name: String,
    ) -> Vec<Implementation> {
        let mut res = Vec::new();
        let mut functions = Vec::new();
        let mut signature = String::new();
        let mut right_impl = false;
        let mut i = index;
        let mut brace_counter = 0;

        while i < tokens.len() {
            match tokens[i] {
                Token::Keyword(Keyword::Impl) => loop {
                    match &tokens[i] {
                        Token::Ident(name) => {
                            if name == &orig_name {
                                right_impl = true;
                            }
                            signature.push_str(&tokens[i].to_string());
                            signature.push_str(" ");
                            i += 1;
                        }
                        Token::LeftBrace => {
                            if !right_impl {
                                signature = "".to_string();
                                break;
                            } else {
                                brace_counter += 1;
                                i += 1;
                                while brace_counter != 0 {
                                    match &tokens[i] {
                                        Token::Keyword(Keyword::Fn) => {
                                            let name = match &tokens[i + 1] {
                                                Token::Ident(idn) => idn.clone(),
                                                _ => {
                                                    i += 1;
                                                    continue;
                                                }
                                            };
                                            let doc = doc(&tokens, i);
                                            let sign = fn_signature(&tokens, i);

                                            functions.push(Function {
                                                name,
                                                doc,
                                                signature: sign,
                                                is_method: true,
                                            });

                                            i += 1;
                                        }
                                        Token::LeftBrace => {
                                            i += 1;
                                            brace_counter += 1;
                                        }
                                        Token::RightBrace => {
                                            i += 1;
                                            brace_counter -= 1;
                                        }
                                        _ => {
                                            i += 1;
                                        }
                                    }
                                }

                                res.push(Implementation {
                                    signature: signature.clone(),
                                    functions: functions.clone(),
                                });
                                signature = "".to_string();
                                functions = vec![];
                                break;
                            }
                        }
                        _ => {
                            signature.push_str(&tokens[i].to_string());
                            signature.push_str(" ");
                            i += 1;
                        }
                    }
                },
                _ => {
                    i += 1;
                }
            }
        }

        res
    }
}

/// Generates an HTML page for a structured code element with associated documentation.

/// The `generate_structure_pages` function generates an HTML page for a structured code element represented by the `structure` parameter.
/// The HTML page displays information about the structured code element, including its name, documentation, signature, additional documentation,
/// and any associated implementations. This function is typically used to generate documentation pages for structured code elements.
fn generate_structure_pages(structure: Structure) -> Result<(), crate::DocError> {
    let rendered_html = structure.render().map_err(|_| crate::DocError::RenderError)?;

    let output_file_name = format!("generated_doc/{}.html", structure.name);

    let mut file = File::create(output_file_name).map_err(|_| crate::DocError::FileCreateError)?;
    file.write_all(rendered_html.as_bytes()).map_err(|_| crate::DocError::FileEditError)?;

    Ok(())
}

/// Represents a trait with associated documentation, methods, and implementations.

/// The `Trait` struct is used to represent a trait, providing information about its name, documentation, signature,
/// additional documentation, required methods, provided methods, and any associated implementations. This structure is
/// designed to organize and process information related to traits within the code.
#[derive(Debug, Template)]
#[template(path = "trait_template.html")]
pub(crate) struct Trait {
    name: String,
    doc: String,
    signature: String,
    additional_doc: String,
    required_methods: Vec<Function>,
    provided_methods: Vec<Function>,
    implementations: Vec<Implementation>,
}

/// Generates an HTML page for a trait with associated documentation, methods, and implementations.

/// The `generate_trait_pages` function generates an HTML page for a trait represented by the `r#trait` parameter.
/// The HTML page displays information about the trait, including its name, documentation, signature, additional documentation,
/// required methods, provided methods, and any associated implementations. This function is typically used to generate
/// documentation pages for traits.
fn generate_trait_pages(r#trait: Trait) -> Result<(), crate::DocError> {
    let rendered_html = r#trait.render().map_err(|_| crate::DocError::RenderError)?;

    let output_file_name = format!("generated_doc/{}.html", r#trait.name);

    let mut file = File::create(output_file_name).map_err(|_| crate::DocError::FileCreateError)?;
    file.write_all(rendered_html.as_bytes()).map_err(|_| crate::DocError::FileEditError)?;

    Ok(())
}

/// Represents a collection of code outputs and their associated filename.

/// The `AllOutput` struct is used to group together a collection of code outputs, typically representing documentation for code elements,
/// and associate them with a specific filename. This structure is commonly used to organize and generate documentation pages that include
/// multiple code outputs from various parts of the codebase.
#[derive(Debug, Template)]
#[template(path = "doc_template.html")]
pub(crate) struct AllOutput {
    pub(crate) all_output: Vec<Output>,
    pub(crate) filename: String,
}

/// Represents search results containing a list of code outputs.

/// The `SearchResults` struct is used to represent search results containing a list of code outputs. This structure is typically used
/// when generating search result pages, allowing users to view and navigate code outputs matching their search queries.
#[derive(Debug, Template)]
#[template(path = "search_results_template.html")]
pub(crate) struct SearchResults {
    results: Vec<Output>,
}

/// Generates an HTML page for displaying search results.

/// The `generate_search_page` function generates an HTML page to display search results provided in the `res` parameter.
/// The generated page includes a list of code outputs that match the search criteria. This function is typically used to
/// create search result pages for users searching for code documentation.
fn generate_search_page(res: SearchResults, module_name: String) -> Result<(), crate::DocError> {
    let rendered_html = res.render().map_err(|_| crate::DocError::RenderError)?;

    let filename = format!("generated_doc/search_results_{}.html", module_name);

    let mut file = File::create(filename).map_err(|_| crate::DocError::FileCreateError)?;
    file.write_all(rendered_html.as_bytes()).map_err(|_| crate::DocError::FileEditError)?;

    Ok(())
}

/// Extracts the filename from a path, given the full filename with its path.

/// The `extract_filename` function takes a full filename with its path as input and extracts the filename portion, excluding the path.
/// This function is useful when you need to isolate the filename from a file path, which can be particularly helpful when generating documentation
/// pages or handling file-related operations.
pub(crate) fn extract_filename(filename_with_path: &str) -> Option<&str> {
    Path::new(filename_with_path).file_stem().and_then(OsStr::to_str)
}

/// Generates an HTML documentation page for a module, including associated code elements.

/// The `generate_module_page` function generates an HTML documentation page for a module and its associated code elements.
/// This page typically includes code outputs, such as functions, structs, traits, and other code constructs. It also provides
/// links to related code documentation and a search feature for easier navigation.
pub(crate) fn generate_module_page(module: AllOutput) -> Result<(), crate::DocError> {
    let rendered_html = module.render().map_err(|_| crate::DocError::RenderError)?;

    let fname = format!("generated_doc/{}.html", module.filename);

    let mut file = File::create(fname).map_err(|_| crate::DocError::FileCreateError)?;
    file.write_all(rendered_html.as_bytes()).map_err(|_| crate::DocError::FileEditError)?;

    let fname = format!("input_files/{}.nr", module.filename);

    if fs::metadata(&fname).is_ok() {
        generate_code_page(&fname)?;
    }

    let res = SearchResults { results: module.all_output.clone() };

    generate_search_page(res, module.filename)?;

    for i in module.all_output.iter() {
        match i.r#type {
            Type::Function => {
                generate_function_pages(Function {
                    name: i.name.clone(),
                    doc: i.doc.clone(),
                    signature: i
                        .information
                        .get_signature()
                        .ok_or(crate::DocError::GetInfoError)?,
                    is_method: false,
                })?;
            }
            Type::Struct => {
                generate_structure_pages(Structure {
                    name: i.name.clone(),
                    doc: i.doc.clone(),
                    additional_doc: i
                        .information
                        .get_additional_doc()
                        .ok_or(crate::DocError::GetInfoError)?,
                    signature: i
                        .information
                        .get_signature()
                        .ok_or(crate::DocError::GetInfoError)?,
                    implementations: i
                        .information
                        .get_implementations()
                        .ok_or(crate::DocError::GetInfoError)?,
                })?;
            }
            Type::Trait => {
                generate_trait_pages(Trait {
                    name: i.name.clone(),
                    doc: i.doc.clone(),
                    signature: i
                        .information
                        .get_signature()
                        .ok_or(crate::DocError::GetInfoError)?,
                    additional_doc: i
                        .information
                        .get_additional_doc()
                        .ok_or(crate::DocError::GetInfoError)?,
                    required_methods: i
                        .information
                        .get_required_methods()
                        .ok_or(crate::DocError::GetInfoError)?,
                    provided_methods: i
                        .information
                        .get_provided_methods()
                        .ok_or(crate::DocError::GetInfoError)?,
                    implementations: i
                        .information
                        .get_implementations()
                        .ok_or(crate::DocError::GetInfoError)?,
                })?;
            }
            Type::Module => {
                generate_module_page(AllOutput {
                    all_output: i.information.get_content().ok_or(crate::DocError::GetInfoError)?,
                    filename: i.name.clone(),
                })?;
            }
            _ => {}
        }
    }

    Ok(())
}
