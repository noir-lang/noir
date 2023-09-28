// Test some empty blocks.
fn qux() {
    {}

    { /* a block with a comment */ }
    {

    }
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
