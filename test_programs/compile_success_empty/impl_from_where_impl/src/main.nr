// use dep::std::cmp::Ordering;

trait Bar {
    fn ok(self) -> Self;

    fn ref_ok(self) -> Self {
        self.ok()
    }
}

impl<T> Bar for (T, T) where T: Bar {
    fn ok(self) -> Self {
        self
    }
}

// trait Bar {
//     fn ok(self) -> Self;
// }
//
// trait Qux {
//     fn id(self) -> Self {
//         self
//     }
// }
//
// // impl<T> Bar for (T, T) where T: Bar {
// impl<T> Bar for (T, T) where T: Bar {
//     fn ok(self) -> Self {
//         // self
//         (self.0.ok(), self.1)
//     }
// }

// fn ref_ok(self) -> Self {
//     self.ok()
// }

// struct Foo<U> {
//     value: U
// }
//
// impl<T> Bar for Foo<T> where T: Bar {
//     fn ok(self) -> Self {
//         self
//     }
//
// }

// impl<T> Bar for Foo<T> where T: Bar {
//     fn ok(self) -> Self {
//         self
//     }
// }


// This works:
//
// trait Bar {
//     fn ok(self) -> Self;
//
//     fn ref_ok(self) -> Self;
// }
//
// impl<T> Bar for Foo<T> where T: Bar {
//     fn ok(self) -> Self {
//         self
//     }
//
//     fn ref_ok(self) -> Self {
//         self.ok()
//     }
// }



// trait Ord2 {
//     fn cmp2(self, other: Self) -> Ordering;
//
//     fn max2(self, rhs: Self) -> Self {
//         if self.cmp2(rhs) == Ordering::greater() { self } else { rhs }
//     }
// }
//
// impl<T> Ord2 for Foo<T> where T: Ord2 {
//     fn cmp2(_self: Self, _other: Self) -> Ordering {
//         Ordering::greater()
//     }
// }

fn main() {
    // let foo: Foo<u32> = Foo { value: 7 };
    // let _ = foo.cmp2(foo);
}

// impl Ord2 for u32 {
//     fn cmp2(_self: Self, _other: Self) -> Ordering {
//         Ordering::equal()
//     }
// }

// // trait Ord {
// //     fn cmp(self, other: Self) -> Ordering;
// //
// //     fn max(self, other: Self) -> Self {
// //         max(self, other)
// //     }
// // }
//
// use dep::std::cmp::Ordering;
//
// trait Ord2 {
//     fn cmp2(self, other: Self) -> Ordering;
//
//     fn max2(self, other: Self) -> Self;
// }
//
//
// fn max2<T>(lhs: T, rhs: T) -> T where T: Ord2 {
//     if lhs.cmp2(rhs) == Ordering::greater() { lhs } else { rhs }
// }
//
//
// struct Foo<T> {
//     value: T
// }
//
// impl<T> Ord2 for Foo<T> where T: Ord2 {
//     fn cmp2(self, other: Self) -> Ordering {
//         Ordering::greater()
//     }
//
//     fn max2(self, other: Self) -> Self {
//         if self.cmp2(other) == Ordering::greater() { self } else { other }
//     }
// }
//
//
//
// fn max<T>(lhs: T, rhs: T) -> T where T: Ord {
//     if lhs > rhs { lhs } else { rhs }
// }
//
// fn main(x: Field, y: pub Field) {
//     assert(x != y);
// }

