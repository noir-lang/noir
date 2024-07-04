mod foo {
    global FOO = 1;
}

use foo::FOO;

fn main() {
    let _ = foo::FOO;
    let _ = FOO;
}
