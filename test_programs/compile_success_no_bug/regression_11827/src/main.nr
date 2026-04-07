comptime fn foo(f: FunctionDefinition) -> Quoted {
    let body = f.body();
    let body = body.modify(|_| Option::none());
    f.set_body(body);
    quote { }
}

struct Spam {
    eggs: u32,
}

fn main() {
    let mut spams = @[Spam { eggs: 0 }];
    bar(&mut spams);
}

#[foo]
fn bar(spams: &mut [Spam]) {
    spams[0].eggs += 1;
}
