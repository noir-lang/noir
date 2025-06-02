trait Trait {
    type T;
}

impl Trait<T = i32> for bool {
}

fn main() {}