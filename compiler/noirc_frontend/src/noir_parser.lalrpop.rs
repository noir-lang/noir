use noirc_errors::Span;

use crate::lexer::token::Tok;
use crate::lexer::token as noir_token;
use crate::lexer::errors::LexerErrorKind;
use crate::parser::TopLevelStatement;
use crate::{Ident, Path, PathKind, UseTree, UseTreeKind};

use lalrpop_util::ErrorRecovery;

grammar<'input, 'err>(input: &'input str, errors: &'err mut [ErrorRecovery<usize, Tok<'input>, &'static str>]);

extern {
    type Location = usize;

    type Error = LexerErrorKind;

    // NOTE: each token needs a terminal defined
    enum Tok<'input> {
        string => Tok::Str(<&'input str>),
        ident => Tok::Ident(<&'input str>),

        // symbols
        "<" => Tok::Less,
        "<=" => Tok::LessEqual,
        ">" => Tok::Greater,
        ">=" => Tok::GreaterEqual,
        "==" => Tok::Equal,
        "!=" => Tok::NotEqual,
        "+" => Tok::Plus,
        "-" => Tok::Minus,
        "*" => Tok::Star,
        "/" => Tok::Slash,
        "%" => Tok::Percent,
        "&" => Tok::Ampersand,
        "^" => Tok::Caret,
        "<<" => Tok::ShiftLeft,
        ">>" => Tok::ShiftRight,
        "." => Tok::Dot,
        ".." => Tok::DoubleDot,
        "(" => Tok::LeftParen,
        ")" => Tok::RightParen,
        "{" => Tok::LeftBrace,
        "}" => Tok::RightBrace,
        "[" => Tok::LeftBracket,
        "]" => Tok::RightBracket,
        "->" => Tok::Arrow,
        "|" => Tok::Pipe,
        "#" => Tok::Pound,
        "," => Tok::Comma,
        ":" => Tok::Colon,
        "::" => Tok::DoubleColon,
        ";" => Tok::Semicolon,
        "!" => Tok::Bang,
        "=" => Tok::Assign,
        // keywords
        "as" => Tok::Keyword(noir_token::Keyword::As),
        "assert" => Tok::Keyword(noir_token::Keyword::Assert),
        "assert_eq" => Tok::Keyword(noir_token::Keyword::AssertEq),
        "bool" => Tok::Keyword(noir_token::Keyword::Bool),
        "break" => Tok::Keyword(noir_token::Keyword::Break),
        "call_data" => Tok::Keyword(noir_token::Keyword::CallData),
        "char" => Tok::Keyword(noir_token::Keyword::Char),
        "comptime" => Tok::Keyword(noir_token::Keyword::CompTime),
        "constrain" => Tok::Keyword(noir_token::Keyword::Constrain),
        "continue" => Tok::Keyword(noir_token::Keyword::Continue),
        "contract" => Tok::Keyword(noir_token::Keyword::Contract),
        "crate" => Tok::Keyword(noir_token::Keyword::Crate),
        "dep" => Tok::Keyword(noir_token::Keyword::Dep),
        "distinct" => Tok::Keyword(noir_token::Keyword::Distinct),
        "else" => Tok::Keyword(noir_token::Keyword::Else),
        "Field" => Tok::Keyword(noir_token::Keyword::Field),
        "fn" => Tok::Keyword(noir_token::Keyword::Fn),
        "for" => Tok::Keyword(noir_token::Keyword::For),
        "fmtstr" => Tok::Keyword(noir_token::Keyword::FormatString),
        "global" => Tok::Keyword(noir_token::Keyword::Global),
        "if" => Tok::Keyword(noir_token::Keyword::If),
        "impl" => Tok::Keyword(noir_token::Keyword::Impl),
        "in" => Tok::Keyword(noir_token::Keyword::In),
        "let" => Tok::Keyword(noir_token::Keyword::Let),
        "mod" => Tok::Keyword(noir_token::Keyword::Mod),
        "mut" => Tok::Keyword(noir_token::Keyword::Mut),
        "pub" => Tok::Keyword(noir_token::Keyword::Pub),
        "quote" => Tok::Keyword(noir_token::Keyword::Quote),
        "return" => Tok::Keyword(noir_token::Keyword::Return),
        "return_data" => Tok::Keyword(noir_token::Keyword::ReturnData),
        "str" => Tok::Keyword(noir_token::Keyword::String),
        "struct" => Tok::Keyword(noir_token::Keyword::Struct),
        "trait" => Tok::Keyword(noir_token::Keyword::Trait),
        "type" => Tok::Keyword(noir_token::Keyword::Type),
        "unchecked" => Tok::Keyword(noir_token::Keyword::Unchecked),
        "unconstrained" => Tok::Keyword(noir_token::Keyword::Unconstrained),
        "use" => Tok::Keyword(noir_token::Keyword::Use),
        "where" => Tok::Keyword(noir_token::Keyword::Where),
        "while" => Tok::Keyword(noir_token::Keyword::While),
        // bool
        "true" => Tok::Bool(true),
        "false" => Tok::Bool(false),

        r"[\t\r\n ]+" => Tok::Whitespace(_),

        EOF => Tok::EOF,
    }
}

pub Grammar: Vec<Vec<Vec<Result<Tok<'input>, Ident>>>> =
    <uses:Use*>
    ";"
    <items:GrammarItem*> => {
        uses.into_iter().chain(items).collect()
    };

GrammarItem: Vec<Vec<Result<Tok<'input>, Ident>>> = {
    Use,
    Nonterminal
};

Use: Vec<Vec<Result<Tok<'input>, Ident>>> =
    <u:"use"> ";" => vec![vec![Ok(u)]];

// TODO what's the '='?
Nonterminal: Vec<Vec<Result<Tok<'input>, Ident>>> =
    <lo:@L> <n:NonterminalName> <hi:@R>
    "=" <a:Alternatives> => {
        let o: Vec<_> = std::iter::once(vec![n]).chain(a.into_iter()).map(|x| x.into_iter().map(Result::Err).collect::<Vec<_>>()).collect();
        o
    };

NonterminalName: Ident = {
    <n:Ident> => n,
};

Alternatives: Vec<Vec<Ident>> = {
    <a:Alternative> ";" => vec![a],
    "{" <Comma<Alternative>> "}" ";",
};

Alternative: Vec<Ident> = {
    // <lo:@L> <s:Symbol+> <c:("if" <Cond>)?> <a:Action?> <hi:@R> => {
    <lo:@L> <s:Ident+> <hi:@R> => s,
};

// Path: Path =
//     <a:"::"?> <h:(<Ident> "::")*> <t:Ident> => {
//         (a, h, t)
//         // Path { absolute: a.is_some(),
//         //        ids: h.into_iter().chain(once(t)).collect() }
//     };

Terminal: Ident = {
    <i:Ident> => i,
};

Comma<E>: Vec<E> =
    <v0:(<E> ",")*> <e1:E?> =>
        v0.into_iter().chain(e1).collect();

Ident: Ident = {
    <lo:@L> <i:ident> <hi:@R> => {
        let token = noir_token::Token::Ident(i.to_string());
        let span = Span::from(lo as u32..hi as u32);
        Ident::from_token(token, span)
    },
}

Bool: Tok<'input> = {
    "true" => Tok::Bool(true),
    "false" => Tok::Bool(false),
};

