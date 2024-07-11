mod foo {
    global FOO = 1;
}

use foo::FOO;

fn main() {
    let _ = foo::FOO;
    let _ = FOO;
    let _: [Field; FOO] = [1];
}

trait WithNumber<N> {

}

struct Some {
}

impl WithNumber<FOO> for Some {

}
