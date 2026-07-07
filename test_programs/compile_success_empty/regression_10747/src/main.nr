// Regression test for https://github.com/noir-lang/noir/issues/10747
//
// A field type obtained via `TypeDefinition::fields_as_written` carries the struct's own generic.
// When spliced via `$typ` into a generated `impl<Context> Storage<Context>`, its `Context` must
// resolve to the generated impl's `Context`, exactly like the textually-written `Context` does.
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

#[storage]
struct Storage<Context> {
    symbol: PublicImmutable<Context>,
}

pub comptime fn storage(s: TypeDefinition) -> Quoted {
    let (name, typ, _) = s.fields_as_written()[0];
    quote {
        impl<Context> Storage<Context> {
            fn init(context: Context) -> Self {
                Self {
                    $name: <$typ as StateVariable<Context>>::new(context),
                }
            }
        }
    }
}

struct PrivateContext {}

fn main() {
    let context: PrivateContext = PrivateContext {};
    let _ = Storage::init(context);
}
