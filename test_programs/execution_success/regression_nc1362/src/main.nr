// Regression for noir-claude#1362: a scalar `array_set` on a composite element
// (here tuple field 0) preserves the element's other fields, including a `&mut`
// reference field (tuple field 1). A store made through that preserved
// reference is observed by a later read of the referenced value.
unconstrained fn set_ref(r: &mut Field) {
    *r = 1;
}

unconstrained fn main() {
    let mut x: Field = 0;
    let r = &mut x;
    let mut arr: [(Field, &mut Field); 1] = [(7, r)];
    arr[0].0 = 99; // scalar array_set on tuple field 0; field 1 keeps `r`
    let rr = arr[0].1; // preserved reference field
    set_ref(rr); // stores 1 through the reference into `x`
    assert(x == 1);
}
