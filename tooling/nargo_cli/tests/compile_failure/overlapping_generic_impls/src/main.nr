
trait Trait { fn t(self); }

impl<T> Trait for T { fn t(self){} }

impl<T> Trait for T { fn t(self){} }

fn main() {}
