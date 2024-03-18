// Modified from https://github.com/rust-lang/rustfmt/blob/master/src/chains.rs


// use noirc_frontend::{token::Token, ArrayLiteral, Literal, UnaryOp};
use noirc_frontend::{token::Token, Expression, ExpressionKind, MethodCallExpression, hir::resolution::errors::Span};

//     ExpressionType, FmtVisitor, Indent, Shape,
// expr::{format_brackets, format_parens, NewlineMode},
use crate::visitor::{
    expr::{format_parens, NewlineMode},
    ExpressionType, FmtVisitor, Shape,
};

pub(crate) fn rewrite(
    visitor: &FmtVisitor,
    expr: Expression,
    _expr_type: ExpressionType,
    shape: Shape,
) -> String {
    match expr.kind {
        ExpressionKind::MethodCall(ref method_call_expr) => {

            let args_span = visitor.span_before(
                method_call_expr.method_name.span().end()..expr.span.end(),
                Token::LeftParen,
            );

            // TODO: remove clone?
            let object = super::sub_expr(visitor, shape, method_call_expr.object.clone());
            let method = method_call_expr.method_name.to_string();

            // TODO: remove clone?
            let args = format_parens(
                visitor.config.fn_call_width.into(),
                visitor.fork(),
                shape,
                false,
                method_call_expr.arguments.clone(),
                args_span,
                true,
                NewlineMode::IfContainsNewLineAndWidth,
            );

            // TODO: remove before PR
            let debug_str: String = Chain::make_subexpr_list(expr).into_iter().map(|x| format!("{:?}\n\n", x)).collect();
            println!("make_subexpr_list:\n{}", debug_str);
            println!();
            println!();
            println!();
            println!("Chain:\n{:?}", Chain::from_ast(expr));

            format!("{object}.{method}{args}")
        }
        _ => unreachable!("method_chain::rewrite called on non-MethodCall ExpressionKind: {:?}", expr.kind)
    }
}

/// Information about an expression in a chain.
#[derive(Debug)]
struct SubExpr {
    expr: Expression,
    is_method_call_receiver: bool,
}

#[derive(Debug)]
struct ChainItem {
    kind: ChainItemKind,
    span: Span,
}

impl ChainItem {
    fn new(expr: &SubExpr) -> ChainItem {
        let (kind, span) =
            ChainItemKind::from_ast(&expr.expr, expr.is_method_call_receiver);
        ChainItem { kind, span }
    }
}


#[derive(Debug)]
enum ChainItemKind {
    Parent {
        expr: Expression,
        parens: bool,
    },
    MethodCall(MethodCallExpression),
    // MethodCall(
    //     ast::PathSegment,
    //     Vec<ast::GenericArg>,
    //     ThinVec<ptr::P<ast::Expr>>,
    // ),
    Comment(Span),
}

impl ChainItemKind {
    fn from_ast(
        expr: &Expression,
        is_method_call_receiver: bool,
    ) -> (ChainItemKind, Span) {
        let kind = match expr.kind {
            ExpressionKind::MethodCall(call) => {
                // TODO: needed for comments??
                //
                // let types = if let Some(ref generic_args) = call.seg.args {
                //     if let ast::GenericArgs::AngleBracketed(ref data) = **generic_args {
                //         data.args
                //             .iter()
                //             .filter_map(|x| match x {
                //                 ast::AngleBracketedArg::Arg(ref generic_arg) => {
                //                     Some(generic_arg.clone())
                //                 }
                //                 _ => None,
                //             })
                //             .collect::<Vec<_>>()
                //     } else {
                //         vec![]
                //     }
                // } else {
                //     vec![]
                // };
                // let span = mk_sp(call.receiver.span.hi(), expr.span.hi());
                // let kind = ChainItemKind::MethodCall(call.seg.clone(), types, call.args.clone());
                let kind = ChainItemKind::MethodCall(call);
                let span = expr.span;
                (kind, span)
            }
            _ => {
                return (
                    ChainItemKind::Parent {
                        expr: expr.clone(),
                        parens: is_method_call_receiver && should_add_parens(expr),
                    },
                    expr.span,
                );
            }
        };

        // TODO: remove comments from the span.
        let start = expr.span.start();
        // let start = context.snippet_provider.span_before(span, ".");
        (kind, Span::new(start, span.end()))
    }
}

#[derive(Debug)]
struct Chain {
    parent: ChainItem,
    children: Vec<ChainItem>,
}

impl Chain {
    fn from_ast(expr: Expression) -> Chain {
        let subexpr_list = Self::make_subexpr_list(expr);

        // Un-parse the expression tree into ChainItems
        let mut rev_children = vec![];

        // TODO remove before PR
        // let mut sub_tries = 0;

        for subexpr in &subexpr_list {
            rev_children.push(ChainItem::new(subexpr));
        }

        // TODO remove before PR
        // fn is_tries(s: &str) -> bool {
        //     s.chars().all(|c| c == '?')
        // }

        // fn is_post_comment(s: &str) -> bool {
        //     let comment_start_index = s.chars().position(|c| c == '/');
        //     if comment_start_index.is_none() {
        //         return false;
        //     }
        //
        //     let newline_index = s.chars().position(|c| c == '\n');
        //     if newline_index.is_none() {
        //         return true;
        //     }
        //
        //     comment_start_index.unwrap() < newline_index.unwrap()
        // }
        //
        // fn handle_post_comment(
        //     post_comment_span: Span,
        //     post_comment_snippet: &str,
        //     prev_span_end: &mut BytePos,
        //     children: &mut Vec<ChainItem>,
        // ) {
        //     let white_spaces: &[_] = &[' ', '\t'];
        //     if post_comment_snippet
        //         .trim_matches(white_spaces)
        //         .starts_with('\n')
        //     {
        //         // No post comment.
        //         return;
        //     }
        //     let trimmed_snippet = trim_tries(post_comment_snippet);
        //     if is_post_comment(&trimmed_snippet) {
        //         children.push(ChainItem::comment(
        //             post_comment_span,
        //             trimmed_snippet.trim().to_owned(),
        //             CommentPosition::Back,
        //         ));
        //         *prev_span_end = post_comment_span.end();
        //     }
        // }

        let parent = rev_children.pop().unwrap();
        let mut children = vec![];
        let mut prev_span_end = parent.span.end();
        let mut iter = rev_children.into_iter().rev().peekable();

        // // TODO needed?
        // if let Some(first_chain_item) = iter.peek() {
        //     let comment_span = Span::inclusive(prev_span_end, first_chain_item.span.start());
        //     let comment_snippet = context.snippet(comment_span);
        //     if !is_tries(comment_snippet.trim()) {
        //         handle_post_comment(
        //             comment_span,
        //             comment_snippet,
        //             &mut prev_span_end,
        //             &mut children,
        //         );
        //     }
        // }

        while let Some(chain_item) = iter.next() {
            // // TODO needed?
            // let comment_snippet = context.snippet(chain_item.span);
            // // FIXME: Figure out the way to get a correct span when converting `try!` to `?`.
            // let handle_comment =
            //     !(context.config.use_try_shorthand() || is_tries(comment_snippet.trim()));
            //
            // // Pre-comment
            // if handle_comment {
            //     let pre_comment_span = mk_sp(prev_span_end, chain_item.span.start());
            //     let pre_comment_snippet = trim_tries(context.snippet(pre_comment_span));
            //     let (pre_comment, _) = extract_pre_comment(&pre_comment_snippet);
            //     match pre_comment {
            //         Some(ref comment) if !comment.is_empty() => {
            //             children.push(ChainItem::comment(
            //                 pre_comment_span,
            //                 comment.to_owned(),
            //                 CommentPosition::Top,
            //             ));
            //         }
            //         _ => (),
            //     }
            // }

            prev_span_end = chain_item.span.end();
            children.push(chain_item);

            // // TODO needed?
            // // Post-comment
            // if !handle_comment || iter.peek().is_none() {
            //     continue;
            // }

            // let next_start = iter.peek().unwrap().span.start();
            // let post_comment_span = mk_sp(prev_span_end, next_start);
            // let post_comment_snippet = context.snippet(post_comment_span);
            // handle_post_comment(
            //     post_comment_span,
            //     post_comment_snippet,
            //     &mut prev_span_end,
            //     &mut children,
            // );
        }

        Chain { parent, children }
    }

    // Returns a Vec of the prefixes of the chain.
    // E.g., for input `a.b.c` we return [`a.b.c`, `a.b`, 'a']
    fn make_subexpr_list(expr: Expression) -> Vec<SubExpr> {
        let mut subexpr_list = vec![SubExpr {
            expr: expr.clone(),
            is_method_call_receiver: false,
        }];

        while let Some(subexpr) = Self::pop_expr_chain(subexpr_list.last().unwrap()) {
            subexpr_list.push(subexpr);
        }

        subexpr_list
    }

    // Returns the expression's subexpression, if it exists.
    // E.g. for a MethodCall, the call's receiver
    fn pop_expr_chain(expr: &SubExpr) -> Option<SubExpr> {
        match &expr.expr.kind {
            ExpressionKind::MethodCall(method_call_expr) => Some(SubExpr {
                expr: method_call_expr.object.clone(),
                is_method_call_receiver: true,
            }),
            _ => None,
        }
    }
}

/// Whether a method call's receiver needs parenthesis, like
/// ```rust,ignore
/// || .. .method();
/// || 1.. .method();
/// 1. .method();
/// ```
/// Which all need parenthesis or a space before `.method()`.
fn should_add_parens(expr: &Expression) -> bool {
    match expr.kind {
        ExpressionKind::Cast(..) => true,
        ExpressionKind::Infix(..) => true,
        ExpressionKind::If(..) => true,
        _ => false,
    }
}


