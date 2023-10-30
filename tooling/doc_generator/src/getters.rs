use std::{fs::File, io::{BufReader, BufRead, Read}};

use askama::Template;
use noirc_frontend::{lexer::Lexer, token::{SpannedToken, Token, DocComments, Keyword}, hir::resolution::errors::Span};

use crate::{Function, Output};

pub(crate) fn get_module_content(tokens: &[Token], index: usize) -> Vec<Output> {
    let mut content = Vec::new();
    let mut i = index;
    let mut brace_counter = 0;

    

    loop {
        match &tokens[i] {
            Token::Semicolon => {
                let filename = format!("input_files/{}.nr", tokens[i - 1]);
                content = get_doc(&filename).unwrap();
                break;
            }
            Token::LeftBrace => {
                brace_counter += 1;
                i += 1;
                while brace_counter != 0 {
                    match &tokens[i] {
                        Token::LeftBrace => {
                            brace_counter += 1;
                            content.push(SpannedToken::new(tokens[i].clone(), Span::inclusive(0, 1)));
                            i += 1;
                        }
                        Token::RightBrace => {
                            brace_counter -= 1;
                            content.push(SpannedToken::new(tokens[i].clone(), Span::inclusive(0, 1)));
                            i += 1;
                        }
                        _ => {
                            content.push(SpannedToken::new(tokens[i].clone(), Span::inclusive(0, 1)));
                            i += 1;
                        }
                    }
                }
                break;
            }
            _ => {
                i += 1;
            }
        };
    }

    let res = Output::to_output(content);

    res
}

pub(crate) fn skip_impl_block(tokens: &[Token], index: usize) -> usize {
    let mut brace_counter = 0;
    let mut i = index;

    while brace_counter != 1 {
        match &tokens[i] {
            Token::LeftBrace => {
                i += 1;
                brace_counter += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    while brace_counter != 0 {
        match &tokens[i] {
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

    i - index - 1
}

pub(crate) fn fn_signature(tokens: &[Token], index: usize) -> String {
    let mut res = String::new();
    let mut i = index;
    loop {
        match &tokens[i] {
            Token::LeftBrace | Token::Semicolon => {
                break;
            }
            _ => {
                res.push_str(&tokens[i].to_string());
                res.push_str(" ");
                i += 1;
            }
        };
    }
    res
}

pub(crate) fn struct_signature(tokens: &[Token], index: usize) -> String {
    let mut res = String::new();
    let mut i = index;
    let mut is_private = true;

    loop {
        match &tokens[i + 1] {
            Token::LeftBrace => {
                res.push_str("{");
                res.push_str("\n");
                loop {
                    match tokens[i + 1] {
                        Token::RightBrace => {
                            if is_private {
                                res.push_str("/* private fields */");
                            }
                            res.push_str("\n");
                            res.push_str("}");
                            break;
                        }
                        Token::Keyword(Keyword::Pub) => {
                            is_private = false;
                            loop {
                                match tokens[i + 1] {
                                    Token::Comma => {
                                        if tokens[i + 2] == Token::RightBrace {
                                            res.push_str(",");
                                        }
                                        else {
                                            res.push_str(",\n");
                                        }
                                        i += 1;
                                        break;
                                    }
                                    Token::RightBrace => {
                                        break;
                                    }
                                    _ => {
                                        res.push_str(&tokens[i + 1].to_string());
                                        res.push_str(" ");
                                        i += 1;
                                    }
                                }
                            }
                        }
                        _ => { i += 1; }
                    }
                }
                break;
            }
            _ => {
                res.push_str(&tokens[i + 1].to_string());
                res.push_str(" ");
                i += 1;
            }
        };
    }

    res
}

pub(crate) fn trait_info(tokens: &[Token], index: usize) -> (String, Vec<Function>, Vec<Function>) {
    let mut sign = String::new();
    let mut required_methods = Vec::new();
    let mut provided_methods = Vec::new();
    let mut i = index;
    let mut brace_counter;

    loop {
        match &tokens[i + 1] {
            Token::LeftBrace => {
                sign.push_str("{");
                sign.push_str("\n");
                loop {
                    match tokens[i + 1] {
                        Token::RightBrace => {
                            sign.push_str("}");
                            break;
                        }
                        Token::Keyword(Keyword::Fn) => {
                            let name = match &tokens[i + 2] {
                                Token::Ident(idn) => {
                                    idn.clone()
                                }
                                _ => {break;}
                            };
                            let doc = doc(&tokens, i + 1);
                            let fn_sign = fn_signature(&tokens, i + 1);

                            loop {
                                match tokens[i + 1] {
                                    Token::Semicolon => {
                                        required_methods.push(Function { name, doc, signature: fn_sign, is_method: true });
                                        sign.push_str(";");
                                        sign.push_str("\n");
                                        break;
                                    }
                                    Token::LeftBrace => {
                                        provided_methods.push(Function { name, doc, signature: fn_sign, is_method: true });
                                        brace_counter = 1;
                                        sign.push_str("{ ... }");
                                        sign.push_str("\n");
                                        while brace_counter != 0 {
                                            i += 1;
                                            match tokens[i + 1] {
                                                Token::LeftBrace => {
                                                    brace_counter +=1;
                                                }
                                                Token::RightBrace => {
                                                    brace_counter -=1;
                                                }
                                                _ => {}
                                            }
                                        }
                                        i +=1;
                                        break;
                                    }
                                    _ => {
                                        sign.push_str(&tokens[i + 1].to_string());
                                        sign.push_str(" ");
                                        i += 1;
                                    }
                                }
                            }
                        }
                        _ => { i += 1; }
                    }
                }
                break;
            }
            _ => {
                sign.push_str(&tokens[i + 1].to_string());
                sign.push_str(" ");
                i += 1;
            }
        };
    }

    (sign, required_methods, provided_methods)
}

pub(crate) fn additional_doc(tokens: &[Token], index: usize) -> String {
    let res = match &tokens[index - 1] {
        Token::DocComment(DocComments::Outer(dc)) => {
            let mut res = dc.to_string();
            let mut doc_end = true;
            let mut iter = 2;
            while doc_end && ((index as i32) - (iter as i32)) >= 0 {
                match &tokens[index - iter] {
                    Token::DocComment(DocComments::Outer(doc)) => {
                        res.insert_str(0, &doc.to_string());
                        iter += 1;
                    }
                    _ => {
                        doc_end = false;
                    }
                }
            }
            res
        }
        _ => {
            let mut res = String::new();
        
            let mut doc_find = true;
            let mut iter = 2;
            while doc_find && ((index as i32) - (iter as i32)) >= 0 {
                match &tokens[index - iter] {
                    Token::DocComment(DocComments::Outer(doc)) => {
                        res.insert_str(0, &doc.to_string());
                        iter += 1;
                    }
                    Token::Keyword(Keyword::Fn) | Token::Keyword(Keyword::Mod) |
                    Token::Keyword(Keyword::Struct) | Token::Keyword(Keyword::Trait) |
                    Token::Keyword(Keyword::Impl) => {
                        doc_find = false;
                    }
                    _ => { iter += 1; }
                }
            }
            res
        }
    };
    res
}

pub(crate) fn doc(tokens: &[Token], index: usize) -> String {
    if index == 0 {
        return String::new();
    }
    let res = match &tokens[index - 1] {
        Token::DocComment(DocComments::Single(dc)) | 
        Token::DocComment(DocComments::Block(dc)) => {
            let mut res = dc.to_string();
            let mut doc_end = true;
            let mut iter = 2;
            while doc_end && ((index as i32) - (iter as i32)) >= 0 {
                match &tokens[index - iter] {
                    Token::DocComment(DocComments::Single(doc)) | 
                    Token::DocComment(DocComments::Block(doc)) => {
                        res.insert_str(0, &doc.to_string());
                        iter += 1;
                    }
                    _ => {
                        doc_end = false;
                    }
                }
            }
            res
        }
        _ => {
            let mut res = String::new();
        
            let mut doc_find = true;
            let mut iter = 2;
            while doc_find && ((index as i32) - (iter as i32)) >= 0 {
                match &tokens[index - iter] {
                    Token::DocComment(DocComments::Single(doc)) | 
                    Token::DocComment(DocComments::Block(doc)) => {
                        res.insert_str(0, &doc.to_string());
                        iter += 1;
                    }
                    Token::Keyword(Keyword::Fn) | Token::Keyword(Keyword::Mod) |
                    Token::Keyword(Keyword::Struct) | Token::Keyword(Keyword::Trait) |
                    Token::Keyword(Keyword::Impl) => {
                        doc_find = false;
                    }
                    _ => { iter += 1; }
                }
            }
            res
        }
    };
    res
}

pub(crate) fn outer_doc(tokens: &[Token], index: usize) -> (String, usize) {
    let mut i = index;
    let mut res = tokens[i].to_string();
    let mut doc_find = true;
    while doc_find {
        match &tokens[i + 1] {
            Token::DocComment(DocComments::Outer(doc)) => {
                res.push_str(doc);
                i += 1;
            }
            _ => { doc_find = false; }
        }
    }

    (res, i)
}

pub(crate) fn get_doc(input_file: &str) -> Result<Vec<SpannedToken>, Box<dyn std::error::Error>> {
    let mut file = File::open(input_file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut lexer = Lexer::new(&contents);
    lexer = lexer.skip_comments(false);

    let token = lexer.into_iter().map(|a| a.unwrap()).collect::<Vec<_>>();

    Ok(token)
}

#[derive(Template)]
#[template(path = "code_template.html")]
pub(crate) struct Code {
    pub(crate) codelines: Vec<CodeLine>,
}

#[derive(Debug)]
pub(crate) struct CodeLine {
    number: u32,
    text: String,
}

pub(crate) fn get_text(input_file: &str) -> Result<Vec<CodeLine>, Box<dyn std::error::Error>> {
    let file = File::open(input_file)?;
    let reader = BufReader::new(file);
    let mut code = Vec::new();
    let mut i = 0;

    for line in reader.lines() {
        i += 1;
        code.push(CodeLine{ number: i, text: line? });
    }

    Ok(code)
}
