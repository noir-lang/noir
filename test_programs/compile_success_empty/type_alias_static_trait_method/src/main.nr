// Regression test for https://github.com/noir-lang/noir-claude/issues/1360
// A type alias to a built-in type with generic arguments (e.g. `str<N>`, `[T; N]`,
// `[T]`, `(A, B)`, `fmtstr<N, T>` or even `fn() -> T`) used to drop those generic
// arguments when dispatching a static trait method through the alias, causing
// "Type annotation needed" at monomorphization.
trait PrimitiveTag {
    fn tag() -> u32;
    fn via_self() -> u32;
}

impl<let N: u32> PrimitiveTag for str<N> {
    fn tag() -> u32 {
        N
    }

    fn via_self() -> u32 {
        Self::tag()
    }
}

impl<let N: u32, T> PrimitiveTag for fmtstr<N, T> {
    fn tag() -> u32 {
        N
    }

    fn via_self() -> u32 {
        Self::tag()
    }
}

impl<T, let N: u32> PrimitiveTag for [T; N] {
    fn tag() -> u32 {
        N
    }

    fn via_self() -> u32 {
        Self::tag()
    }
}

impl<T> PrimitiveTag for [T] {
    fn tag() -> u32 {
        7
    }

    fn via_self() -> u32 {
        Self::tag()
    }
}

impl<A, B> PrimitiveTag for (A, B) {
    fn tag() -> u32 {
        2
    }

    fn via_self() -> u32 {
        Self::tag()
    }
}

impl<T> PrimitiveTag for fn() -> T {
    fn tag() -> u32 {
        8
    }

    fn via_self() -> u32 {
        Self::tag()
    }
}

type Str3 = str<3>;
type Str5 = str<5>;
type Fmt3 = fmtstr<3, (Field, Field)>;
type Arr = [u8; 4];
type Sli = [u8];
type Tup = (u8, u32);
type Func = fn() -> u8;

fn main() {
    assert(Str3::tag() == 3);
    assert(Str5::tag() == 5);
    assert(Str3::via_self() == 3);
    assert(Str5::via_self() == 5);

    // Fully-qualified syntax was the documented workaround and must keep working.
    assert(<Str3 as PrimitiveTag>::tag() == 3);

    assert(Fmt3::tag() == 3);
    assert(Fmt3::via_self() == 3);

    assert(Arr::tag() == 4);
    assert(Arr::via_self() == 4);

    assert(Sli::tag() == 7);
    assert(Sli::via_self() == 7);

    assert(Tup::tag() == 2);
    assert(Tup::via_self() == 2);

    // A generic function type: the impl's `T` is bound from the alias even though the
    // method body never mentions it. The structural-args approach couldn't handle this
    // (a function type's components aren't its generic arguments); unifying the whole
    // self type against the impl's self type does.
    assert(Func::tag() == 8);
    assert(Func::via_self() == 8);
}
