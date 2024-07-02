mod foo {
    pub fn some_function() -> Field {
        1 + 2
    }
}

use foo::some_function;

fn main() {
    let _ = some_function();
}

