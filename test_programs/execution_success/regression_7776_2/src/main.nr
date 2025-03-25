fn main(a: Field, b: Field) -> pub i8 {
  let lhs = sub_and_truncate(a, b);
  // Safety: Test program
  let rhs = unsafe {__sub_and_truncate(a, b)};
  assert_eq(lhs, rhs);
  lhs
}

fn sub_and_truncate(a: Field, b: Field) -> i8 {
  ((a - b) as i8)
}

unconstrained fn __sub_and_truncate(a: Field, b: Field) -> i8 {
  ((a - b) as i8)
}
