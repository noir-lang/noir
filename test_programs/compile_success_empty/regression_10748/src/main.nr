pub trait StateVariable<Context> {
    fn new(context: Context) -> Self;
}

pub struct PublicImmutable<Context> {
    context: Context,
}

impl<Context> StateVariable<Context> for PublicImmutable<Context> {
    fn new(context: Context) -> Self {
        PublicImmutable { context }
    }
}

pub comptime fn storage(s: TypeDefinition) -> Quoted {
    let (_name, typ, _) = s.fields_as_written()[0];

    let context = std::meta::typ::fresh_type_variable();
    assert(
        typ.implements(quote { crate::StateVariable<$context> }.as_trait_constraint()),
        f"Type {typ} was placed in Storage struct but does not implement the StateVariable trait",
    );

    quote {
        impl Storage<$context> { }
    }
}

#[storage]
struct Storage<Context> {
    symbol: PublicImmutable<Context>,
}

impl<Context> Storage<Context> {
    pub fn init(context: Context) -> Self {
        Storage {
            symbol: PublicImmutable::new(context)
        }
    }
}

struct PrivateContext {}

fn main() {
    let context: PrivateContext = PrivateContext {};
    let _s = Storage::init(context);
}