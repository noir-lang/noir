use super::*;

pub struct CastParser;

impl CastParser {
    pub fn parse(parser: &mut Parser, lhs: Expression) -> ParserExprKindResult {
        parser.advance_tokens();

        let typ = parser.parse_type()?;

        Ok(ExpressionKind::Cast(Box::new(CastExpression { lhs: lhs, r#type: typ })))
    }
}
