#[builtin(array_len)]
fn len<T>(_input : [T]) -> comptime Field {}

#[builtin(arraysort)]
 fn sort<T>(_a: [T]) -> [T] {}
