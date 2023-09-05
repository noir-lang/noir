#[oracle(computeSelector)]
fn compute_selector_oracle<T>(_selector: T, _size: Field) -> Field {}

unconstrained fn compute_selector<T>(signature: T) -> Field {
  compute_selector_oracle(signature, 0)
}