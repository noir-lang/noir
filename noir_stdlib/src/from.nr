trait From<T> {
    fn from(input: T) -> Self;
}

trait Into<T> {
    fn into(input: Self) -> T;
}

impl<T, U> Into<T> for U where T: From<U> {
  fn into(input: U) -> T {
      T::from(input)
  }
}
