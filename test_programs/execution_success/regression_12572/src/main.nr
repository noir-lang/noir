use std::default::Default;

#[derive(Default)]
struct Log<let N: u32> {
    fields: [Field; N],
}

#[derive(Default)]
struct Wrapper {
    log: Log<2>,
}

#[derive(Default)]
struct Bundle {
    small: [Log<1>; 1],
    wrapped: [Wrapper; 1],
}

fn main(input: Field) -> pub Field {
    let mut b: Bundle = Default::default();
    b.wrapped[0].log.fields[0] = input;
    b.wrapped[0].log.fields[0] + b.wrapped[0].log.fields[1] + b.small[0].fields[0]
}
