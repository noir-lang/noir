// Regression for issue #5212 (https://github.com/noir-lang/noir/issues/5212)
fn main() {
    let lambda1 = || {
        1
    };

    // Compiler panic when you create a variable in a lambda with another lambda call as its value
    let lambda2 = || {
        let var2: u8 = lambda1();
    };

    // No panic if the variable is created before we assign its value as the lambda call
    let lambda3 = || {
        let mut var3: u8 = 1;
        var3 = lambda1();
    };

    // No panic with a function call that has the same behavior
    let lambda4 = || {
        let var4: u8 = func();
    };

    // No panic outside a lambda code block
    let var5: u8 = lambda1();
}

fn func() -> u8 {
    1
}

