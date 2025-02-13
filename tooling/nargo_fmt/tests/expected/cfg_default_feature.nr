mod foo_module {
    pub global FOO: bool = true;
}

#[cfg(feature = "default")]
use foo_module::FOO;

fn main() {
    let _ = FOO;
}
