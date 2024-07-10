fn some_function() -> Field {
    1 + 2
}

fn another_function() -> Field {
    3 + 4
}

fn main() {
    let _ = another_function();

    let _ = 1;

    let _ = another_function();
}

mod foo {
    fn some_other_function() -> Field {
        crate::another_function()
    }
}

use foo::some_other_function as bar;

fn x() {
    bar();
}
