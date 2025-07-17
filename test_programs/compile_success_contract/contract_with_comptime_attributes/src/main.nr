pub contract SomeContract {
    #[super::add_utility]
    fn fn1() {}

}

comptime fn utility(f: FunctionDefinition) -> () {
    f.add_attribute("'custom")
}

comptime fn add_utility(_: FunctionDefinition) -> Quoted {
    let utility = utility;

    quote {
        #[$utility]
        fn fn2() {
        }
    }
}
