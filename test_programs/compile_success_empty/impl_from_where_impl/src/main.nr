// trait Ord {
//     fn cmp(self, other: Self) -> Ordering;
//
//     fn max(self, other: Self) -> Self {
//         max(self, other)
//     }
// }

use dep::std::cmp::Ordering;

trait Ord2 {
    fn cmp2(self, other: Self) -> Ordering;

    fn max2(self, other: Self) -> Self;
}


fn max2<T>(lhs: T, rhs: T) -> T where T: Ord2 {
    if lhs.cmp2(rhs) == Ordering::greater() { lhs } else { rhs }
}


struct Foo<T> {
    value: T
}

impl<T> Ord2 for Foo<T> where T: Ord2 {
    fn cmp2(self, other: Self) -> Ordering {
        Ordering::greater()
    }

    fn max2(self, other: Self) -> Self {
        if self.cmp2(other) == Ordering::greater() { self } else { other }
    }
}



fn max<T>(lhs: T, rhs: T) -> T where T: Ord {
    if lhs > rhs { lhs } else { rhs }
}

fn main(x: Field, y: pub Field) {
    assert(x != y);
}
