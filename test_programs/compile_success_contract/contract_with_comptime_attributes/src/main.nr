pub contract SomeContract {
    #[super::add_utility]
    fn fn1() {}

}

comptime fn utility(f: FunctionDefinition) -> () {
    println(f.name());
}

comptime fn add_utility(_: FunctionDefinition) -> Quoted {
    let utility = utility;

    quote {
        #[$utility]
        fn fn2() {
        }
    }
}
