#[builtin(array_len)]
fn len<T>(_input : [T]) -> comptime Field {}

#[builtin(arraysort)]
fn sort<T, N>(_a: [T; N]) -> [T; N] {}
