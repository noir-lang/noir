use std::{fs::{File, self}, io::Write, path::Path};

use askama::Template;
use noirc_frontend::token::{Token, Keyword};

use crate::{Type, Output, fn_signature, doc, Code, get_text};

fn generate_code_page(input_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let codelines = get_text(input_file)?;

    let code = Code{ codelines };

    let rendered_html = code.render().unwrap();

    let fname = format!("generated_doc/codepage_{}.html", extract_filename(input_file).unwrap());

    let mut file = File::create(fname)?;
    file.write_all(rendered_html.as_bytes())?;

    Ok(())
}

#[derive(Debug, Clone, Template, Eq, Hash, PartialEq)]
#[template(path = "func_template.html")]
pub(crate) struct Function {
    pub(crate) name: String, 
    pub(crate) doc: String, 
    pub(crate) signature: String,
    pub(crate) is_method: bool,
}

fn generate_function_pages(func: Function) -> Result<(), Box<dyn std::error::Error>> {
    if func.is_method {
        return Ok(());
    }
    let rendered_html = func.render().unwrap();

    let output_file_name = format!("generated_doc/{}.html", func.name);

    let mut file = File::create(output_file_name)?;
    file.write_all(rendered_html.as_bytes())?;

    Ok(())
}

#[derive(Debug, Template)]
#[template(path = "struct_template.html")]
pub(crate) struct Structure {
    name: String, 
    doc: String, 
    additional_doc: String,
    signature: String,
    implementations: Vec<Implementation>,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub(crate) struct Implementation {
    signature: String,
    functions: Vec<Function>,
}

impl Implementation {
    pub(crate) fn get_implementations(tokens: &[Token], index: usize, orig_name: String) -> Vec<Implementation> {
        let mut res = Vec::new();
        let mut functions = Vec::new();
        let mut signature = String::new();
        let mut right_impl = false;
        let mut i = index;
        let mut brace_counter = 0;

        while i < tokens.len() {
            match tokens[i] {
                Token::Keyword(Keyword::Impl) => {
                    loop {
                        match &tokens[i] {
                            Token::Ident(name) => {
                                if name == &orig_name {
                                    right_impl = true;
                                }
                                signature.push_str(&tokens[i].to_string());
                                signature.push_str(" ");
                                i +=1;
                            }
                            Token::LeftBrace => {
                                if !right_impl {
                                    signature = "".to_string();
                                    break;
                                }
                                else {
                                    brace_counter += 1;
                                    i += 1;
                                    while brace_counter != 0 {
                                        match &tokens[i] {
                                            Token::Keyword(Keyword::Fn) => {
                                                let name = match &tokens[i + 1] {
                                                    Token::Ident(idn) => {
                                                        idn.clone()
                                                    }
                                                    _ => {continue;}
                                                };
                                                let doc = doc(&tokens, i);
                                                let sign = fn_signature(&tokens, i);
                                                
                                                functions.push(Function{ name, doc, signature: sign, is_method: true });

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

                                    res.push(Implementation { signature: signature.clone(), functions: functions.clone() });
                                    signature = "".to_string();
                                    functions = vec![];
                                    break;
                                }
                            }
                            _ => {
                                signature.push_str(&tokens[i].to_string());
                                signature.push_str(" ");
                                i +=1;
                            }
                        }
                    }
                }
                _ => {i += 1;}
            }
        }

        res
    }
}

fn generate_structure_pages(structure: Structure) -> Result<(), Box<dyn std::error::Error>> {
    let rendered_html = structure.render().unwrap();

    let output_file_name = format!("generated_doc/{}.html", structure.name);

    let mut file = File::create(output_file_name)?;
    file.write_all(rendered_html.as_bytes())?;

    Ok(())
}

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

fn generate_trait_pages(r#trait: Trait) -> Result<(), Box<dyn std::error::Error>> {
    let rendered_html = r#trait.render().unwrap();

    let output_file_name = format!("generated_doc/{}.html", r#trait.name);

    let mut file = File::create(output_file_name)?;
    file.write_all(rendered_html.as_bytes())?;

    Ok(())
}

#[derive(Debug, Template)]
#[template(path = "doc_template.html")]
pub(crate) struct AllOutput {
    pub(crate) all_output: Vec<Output>,
    pub(crate) filename: String,
}

#[derive(Debug, Template)]
#[template(path = "search_results_template.html")]
pub(crate) struct SearchResults {
    results: Vec<Output>
}

fn generate_search_page(res: SearchResults, module_name: String) -> Result<(), Box<dyn std::error::Error>> {
    let rendered_html = res.render().unwrap();

    let filename = format!("generated_doc/search_results_{}.html", module_name);

    let mut file = File::create(filename)?;
    file.write_all(rendered_html.as_bytes())?;

    Ok(())
}

pub(crate) fn extract_filename(filename_with_path: &str) -> Option<&str> {
    let path = Path::new(filename_with_path);
    match path.file_stem() {
        Some(file_stem) => file_stem.to_str(),
        None => None,
    }
}

pub(crate) fn generate_module_page(module: AllOutput) -> Result<(), Box<dyn std::error::Error>> {
    let rendered_html = module.render().unwrap();

    let fname = format!("generated_doc/{}.html", module.filename);

    let mut file = File::create(fname)?;
    file.write_all(rendered_html.as_bytes())?;

    let fname = format!("input_files/{}.nr", module.filename);

    if fs::metadata(&fname).is_ok() {
        generate_code_page(&fname)?;
    }

    let res = SearchResults{ results: module.all_output.clone() };

    generate_search_page(res, module.filename)?;

    for i in module.all_output.iter() {
        match i.r#type {
            Type::Function => {
                generate_function_pages(
                    Function { 
                        name: i.name.clone(), 
                        doc: i.doc.clone(), 
                        signature: i.information.get_signature().unwrap(),
                        is_method: false, 
                    }
                )?;
            } 
            Type::Struct => {
                generate_structure_pages(
                    Structure { 
                        name: i.name.clone(), 
                        doc: i.doc.clone(), 
                        additional_doc: i.information.get_additional_doc().unwrap(),
                        signature: i.information.get_signature().unwrap(), 
                        implementations: i.information.get_implementations().unwrap()
                    } 
                )?;
            } 
            Type::Trait => {
                generate_trait_pages(
                    Trait { 
                        name: i.name.clone(),
                        doc: i.doc.clone(), 
                        signature: i.information.get_signature().unwrap(), 
                        additional_doc: i.information.get_additional_doc().unwrap(),
                        required_methods: i.information.get_required_methods().unwrap(), 
                        provided_methods: i.information.get_provided_methods().unwrap(), 
                        implementations: i.information.get_implementations().unwrap()
                    }
                )?;
            }
            Type::Module => {
                generate_module_page(
                    AllOutput { 
                        all_output: i.information.get_content().unwrap(), 
                        filename: i.name.clone() 
                    } 
                )?;
            }
            _ => {}
        }
    }

    Ok(())
}
