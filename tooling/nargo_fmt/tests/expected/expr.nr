// Test some empty blocks.
fn qux() {
    {}

    { /* a block with a comment */ }
    {}
    {
        // A block with a comment.
    }

    {
        {
            {
                // A block with a comment.
            }
        }
    }
}

fn foo_return() {
    "yay"
}

fn fooblock() {
    {
        "inner-block"
    }
}

fn fooblock() {
    {
        {
            {
                "inner-block"
            }
        }
    }
}

fn comment() {
    // this is a test comment
    1
}

fn only_comment() {
    // Keep this here
}

fn only_comments() {
    // Keep this here
    // Keep this here
}

fn only_comments() {
    // Keep this here
    // Keep this here
}

fn commnet() {
    1
    //
}

fn test() {
    34
}

fn test() {
    // 324
    34
}

fn line() {
    42; // 42
}

fn line() {
    42; // 42
    42;
    // hello
}

fn line() {
    42; // 42
    // 42
    // hello
}

fn parenthesized() {
    value + (x as Field)
}

fn parenthesized() {
    (i as u8) + (j as u8) + (k as u8) + x + y + z
}

fn parenthesized() {
    value + ( /*test*/ x as Field /*test*/)
}

fn parenthesized() {
    value
        + (
            // line
            x as Field
        )
}

fn constructor() {
    Point { x: 5, y: 10 };
}

fn if_expr() {
    if true {
        println("Hello :D");
    }
}

fn return_if_expr() {
    if true {
        42
    } else {
        40 + 2
    }
}

fn return_if_expr() {
    if true { 42 };

    if true {
        42
    } else {
        40 + 2
    }
}

fn if_if() {
    (if cond { some(); } else { none(); }).bar().baz();
}
