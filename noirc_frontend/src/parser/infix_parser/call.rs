use crate::{Path, PathKind, parser::{errors::ParserErrorKind}};
use noirc_errors::Spanned;
use super::*;

pub struct CallParser;

impl CallParser {
    // XXX: We can probably generalise this Parser as there is no difference now between External and Internal 
    // Maybe also generalise to constants from other namespaces too! see path.rs infix
    // This will wait until we've transitioned to Interned Paths
    //
    // XXX: Currently this is a special case of path.rs infix. We could fix this by either adding Identifier as an enum which may refer 
    // to a path or an Ident. Or we can deprecate Ident and use Path for Identifiers.
    // Ideally, the logic for Path parser, the logic for the Use Parser and the logic for Expressions like func which can 
    // include paths, should be unified. 
    // 
    // TLDR; this function only triggers for local functions already in the namespace and not for external functions
    pub fn parse(parser: &mut Parser, func_name: Expression) -> ParserExprKindResult {
        let arguments = parser.parse_comma_separated_argument_list(Token::RightParen)?;

        let func_name_iden = match func_name.kind {
            ExpressionKind::Ident(x) => Spanned::from(func_name.span, x),
            _=> return Err(ParserErrorKind::UnstructuredError{message: format!("expected an identifier for the function name"), span : func_name.span}.into_err(parser.file_id))
        };
        
        let path_to_func = Path {
            kind : PathKind::Plain,
            segments : vec![func_name_iden.into()],
        };

        let call_expr = CallExpression {
            func_name: path_to_func ,
            arguments,
        };

       Ok(ExpressionKind::Call(NoirPath::Current, Box::new(call_expr)))
    }
}
