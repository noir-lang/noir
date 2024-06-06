contract TestContract {
    use dep::test::module::foo;

    fn constructor(param: Field, pub_param: pub Field) -> pub [Field; 2] {
        [foo::bar(param), param + pub_param]
    }

    fn someFunction() -> pub Field {
        42
    }
}
