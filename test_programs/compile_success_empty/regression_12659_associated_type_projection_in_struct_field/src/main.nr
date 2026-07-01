// Regression test for https://github.com/noir-lang/noir/issues/12659
// Associated type projections in struct-field type position should resolve.

pub trait RuntimeState<Context> {
    type Runtime;
}

pub struct PublicMutable<T> {}

pub struct RuntimePublicMutable<T, Context> {
    pub context: Context,
    pub slot: Field,
}

impl<T, Context> RuntimeState<Context> for PublicMutable<T> {
    type Runtime = RuntimePublicMutable<T, Context>;
}

pub struct ContextType {}

pub struct Storage {
    pub value: <PublicMutable<u128> as RuntimeState<ContextType>>::Runtime,
}

fn main() {
    let _: Storage = Storage { value: RuntimePublicMutable { context: ContextType {}, slot: 0 } };
}
