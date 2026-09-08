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
        .map(|field| {
            let field_type = field.typ();
            quote { <$field_type as Ser>::N }
        })
        .join(quote { + });

    let generics = s
        .generics()
        .map(|generic| {
            let name = generic.typ();
            let maybe_int = generic.numeric_type();
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
        .filter(|generic| generic.numeric_type().is_none())
        .map(|generic| {
            let name = generic.typ();
            quote { $name: Ser }
        })
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
