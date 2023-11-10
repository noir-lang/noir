
fn main() {
    let array: [Field; 3] = [1, 2, 3];
    assert(foo(array));

    // Ensure this still works if we have to infer the type of the integer literals
    let array = [1, 2, 3];
    assert(foo(array));
}

fn foo<T>(x: T) -> bool where T: Eq {
    x.eq(x)
}

trait Eq {
    fn eq(self, other: Self) -> bool;
}

impl<T, N> Eq for [T; N] where T: Eq {
    fn eq(self, other: Self) -> bool {
        let mut ret = true;
        for i in 0 .. self.len() {
            ret &= self[i].eq(other[i]);
        }
        ret
    }
}

// fn main() {
//   let mut ops: [Field] = [1, 10, 2];
//   let mut numbers: [Field] = [1, 2, 4];
//   let mut final = 0;
// 
//   for i in 0..2 {
//     if ops[i] == 10 {
//       // This line seems to cause the error.
//       numbers[i+1] += numbers[i];
//       final = numbers[i+1];
//     }
//   }
// }
// fn main() {
//   let mut a: [Field] = [1, 2, 3];
//   let mut b: [Field] = [4, 5, 6];
// 
//   for i in 0..2 {
//     if a[i] == 2 {
//       // Bug occurs in following line
//       b[i + 1] = b[i] * b[i + 1];
//     }
//   }
// }
// 
// #[test]
// fn test_main() {
//   main();
// }
/// fn main() {
///     foo(&mut 2, [1]);
/// }
// unconstrained fn foo(x: &mut Field, y: [Field]){}

// Speed test:
// use dep::std::collections::vec::Vec;
// 
// fn main() {
//     assert(to_bits(1234) != 5678);
// }
// 
// fn to_bits(self: Self) -> [u1; MAX_BITS] {
//     let mut res = [0 as u1; MAX_BITS];
// 
//     for i in 0..NUM_LIMBS {
//         let limb_bits = (self.limbs[i] as Field).to_le_bits(BITS_PER_LIMB as u32);
//         for j in 0..BITS_PER_LIMB {
//             let idx = i * (BITS_PER_LIMB as Field) + (j as Field);
//             res[idx] = limb_bits[j as Field];
//         }
//     }
// 
//     res
// }
