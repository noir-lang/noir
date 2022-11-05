#[builtin(arraysum)]
fn sum(_input : [Field]) -> Field {}

#[builtin(arrayprod)]
fn prod(_input : [Field]) -> Field {}

#[builtin(arraylen)]
fn len<T>(_input : [T]) -> const Field {}
