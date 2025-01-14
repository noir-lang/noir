fn main() {
    comptime {
        let block = quote {{
            1;
            2;
            3
        }};
        let statements = block.as_expr().unwrap().as_block().unwrap();
        let last = statements.pop_back().1;

        // `3` should  fit in an expression position even though it is
        // originally a StatementKind::Expression
        let regression1 = quote {{
            let _ = $last;
        }};
        assert(regression1.as_expr().is_some());

        // `1;` should  fit in an expression position even though it is
        // originally a StatementKind::Semi
        let first = statements.pop_front().0;
        let regression2 = quote {{
            let _ = $first;
        }};
        assert(regression2.as_expr().is_some());
    }
}
