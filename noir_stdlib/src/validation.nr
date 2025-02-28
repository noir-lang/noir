use crate::hint::black_box;
use crate::meta::derive_via;

#[derive_via(derive_asserts_is_valid_input)]
pub trait AssertsIsValidInput {
    fn assert_is_valid_input(self);
}

comptime fn derive_asserts_is_valid_input(s: StructDefinition) -> Quoted {
    let name = quote { AssertsIsValidInput };
    let signature = quote { fn assert_is_valid_input(self)  };
    let for_each_field = |name| quote { AssertsIsValidInput::assert_is_valid_input(self.$name); };
    let body = |fields| quote {  $fields  };
    crate::meta::make_trait_impl(s, name, signature, for_each_field, quote { }, body)
}

impl AssertsIsValidInput for bool {
    fn assert_is_valid_input(self) {
        black_box(self as Field).assert_max_bit_size::<1>();
    }
}

impl AssertsIsValidInput for u1 {
    fn assert_is_valid_input(self) {
        black_box(self as Field).assert_max_bit_size::<1>();
    }
}

impl AssertsIsValidInput for u8 {
    fn assert_is_valid_input(self) {
        black_box(self as Field).assert_max_bit_size::<8>();
    }
}

impl AssertsIsValidInput for i8 {
    fn assert_is_valid_input(self) {
        black_box(self as Field).assert_max_bit_size::<8>();
    }
}

impl AssertsIsValidInput for u16 {
    fn assert_is_valid_input(self) {
        black_box(self as Field).assert_max_bit_size::<16>();
    }
}

impl AssertsIsValidInput for i16 {
    fn assert_is_valid_input(self) {
        black_box(self as Field).assert_max_bit_size::<16>();
    }
}

impl AssertsIsValidInput for u32 {
    fn assert_is_valid_input(self) {
        black_box(self as Field).assert_max_bit_size::<32>();
    }
}

impl AssertsIsValidInput for i32 {
    fn assert_is_valid_input(self) {
        black_box(self as Field).assert_max_bit_size::<32>();
    }
}

impl AssertsIsValidInput for u64 {
    fn assert_is_valid_input(self) {
        black_box(self as Field).assert_max_bit_size::<64>();
    }
}

impl AssertsIsValidInput for i64 {
    fn assert_is_valid_input(self) {
        black_box(self as Field).assert_max_bit_size::<64>();
    }
}

impl AssertsIsValidInput for u128 {
    fn assert_is_valid_input(self) {
        black_box(self as Field).assert_max_bit_size::<128>();
    }
}

impl AssertsIsValidInput for Field {
    fn assert_is_valid_input(self) {}
}

impl<let N: u32> AssertsIsValidInput for str<N> {
    fn assert_is_valid_input(self) {
        self.as_bytes().assert_is_valid_input();
    }
}

impl AssertsIsValidInput for () {
    fn assert_is_valid_input(self) {}
}

impl<T, let N: u32> AssertsIsValidInput for [T; N]
where
    T: AssertsIsValidInput,
{
    fn assert_is_valid_input(self) {
        for elem in self {
            elem.assert_is_valid_input();
        }
    }
}

impl<T> AssertsIsValidInput for [T]
where
    T: AssertsIsValidInput,
{
    fn assert_is_valid_input(self) {
        for elem in self {
            elem.assert_is_valid_input();
        }
    }
}

impl<A, B> AssertsIsValidInput for (A, B)
where
    A: AssertsIsValidInput,
    B: AssertsIsValidInput,
{
    fn assert_is_valid_input(self) {
        self.0.assert_is_valid_input();
        self.1.assert_is_valid_input();
    }
}

impl<A, B, C> AssertsIsValidInput for (A, B, C)
where
    A: AssertsIsValidInput,
    B: AssertsIsValidInput,
    C: AssertsIsValidInput,
{
    fn assert_is_valid_input(self) {
        self.0.assert_is_valid_input();
        self.1.assert_is_valid_input();
        self.2.assert_is_valid_input();
    }
}

impl<A, B, C, D> AssertsIsValidInput for (A, B, C, D)
where
    A: AssertsIsValidInput,
    B: AssertsIsValidInput,
    C: AssertsIsValidInput,
    D: AssertsIsValidInput,
{
    fn assert_is_valid_input(self) {
        self.0.assert_is_valid_input();
        self.1.assert_is_valid_input();
        self.2.assert_is_valid_input();
        self.3.assert_is_valid_input();
    }
}

impl<A, B, C, D, E> AssertsIsValidInput for (A, B, C, D, E)
where
    A: AssertsIsValidInput,
    B: AssertsIsValidInput,
    C: AssertsIsValidInput,
    D: AssertsIsValidInput,
    E: AssertsIsValidInput,
{
    fn assert_is_valid_input(self) {
        self.0.assert_is_valid_input();
        self.1.assert_is_valid_input();
        self.2.assert_is_valid_input();
        self.3.assert_is_valid_input();
        self.4.assert_is_valid_input();
    }
}
