// Regression test for https://github.com/noir-lang/noir/issues/10747
//
// A `#[derive]`-style macro splices each field type into `<$field_type as Ser>::N` inside the
// generated impl's own `let N: u32` associated constant. Once resolved, the spliced `[T; N]`
// carries the associated-constant projection `<[T; N] as Ser>::N`. Rebinding the free generics of
// a spliced type must leave such projections alone: capturing this projection by name onto the
// generated impl's own `Self::N` would produce the cyclic associated constant `Self::N = N * Self::N`.
pub trait Ser {
    let N: u32;
}

impl Ser for Field {
    let N: u32 = 1;
}

impl<T, let M: u32> Ser for [T; M]
where
    T: Ser,
{
    let N: u32 = <T as Ser>::N * M;
}

#[derive_ser]
struct Wrapped<T, let N: u32> {
    xs: [T; N],
}

pub comptime fn derive_ser(s: TypeDefinition) -> Quoted {
    let typ = s.as_type();
    let rhs = s
        .fields_as_written()
        .map(|(_, field_type, _): (Quoted, Type, Quoted)| quote { <$field_type as Ser>::N })
        .join(quote { + });

    let generics = s
        .generics()
        .map(|(name, maybe_int): (Type, Option<Type>)| {
            if maybe_int.is_some() {
                let int_type = maybe_int.unwrap();
                quote { let $name: $int_type }
            } else {
                quote { $name }
            }
        })
        .join(quote {,});
    let where_clause = s
        .generics()
        .filter(|(_, maybe_int): (Type, Option<Type>)| maybe_int.is_none())
        .map(|(name, _): (Type, Option<Type>)| quote { $name: Ser })
        .join(quote {,});

    quote {
        impl<$generics> Ser for $typ
        where $where_clause
        {
            let N: u32 = $rhs;
        }
    }
}

fn main() {
    let _wrapped: Wrapped<Field, 3> = Wrapped { xs: [1, 2, 3] };
    let _len: u32 = <Wrapped<Field, 3> as Ser>::N;
}
