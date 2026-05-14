// Regression test for https://github.com/noir-lang/noir/issues/12659
// `<ConcreteType as Trait>::AssocItem` projections must resolve in
// struct-field type position, not only in function bodies.

trait RuntimeState<Context> {
    type Runtime;
}

struct PublicMutable<T> {}

struct RuntimePublicMutable<T, Context> {
    context: Context,
    slot: Field,
}

impl<T, Context> RuntimeState<Context> for PublicMutable<T> {
    type Runtime = RuntimePublicMutable<T, Context>;
}

struct ContextType {}

struct Storage {
    value: <PublicMutable<u128> as RuntimeState<ContextType>>::Runtime,
}

trait HasSize {
    let N: u32;
}

impl<T> HasSize for PublicMutable<T> {
    let N: u32 = 1;
}

struct StorageArray {
    value: [Field; <PublicMutable<u128> as HasSize>::N],
}

struct Wrapper<let N: u32> {
    value: [Field; N],
}

struct StorageWrapper {
    value: Wrapper<<PublicMutable<u128> as HasSize>::N>,
}

fn main() {
    let _ = Storage {
        value: RuntimePublicMutable::<u128, ContextType> { context: ContextType {}, slot: 0 },
    };
    let _ = StorageArray { value: [0] };
    let _ = StorageWrapper { value: Wrapper::<1> { value: [0] } };
}
