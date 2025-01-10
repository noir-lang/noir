fn func_name(x: impl Eq) {}

fn func_name<T>(x: impl Eq, y: T)
where
    T: SomeTrait + Eq,
{}
