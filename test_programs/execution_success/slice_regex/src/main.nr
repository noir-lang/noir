struct Match {
    succeeded: bool,
    match_ends: u32,
    leftover: [u8],
}

impl Match {
    fn empty(leftover: [u8]) -> Self {
        Match { succeeded: true, match_ends: 0, leftover }
    }
}

impl Eq for Match {
    fn eq(self, other: Self) -> bool {
        (self.succeeded == other.succeeded) & (self.match_ends == other.match_ends)
        // (self.leftover == other.leftover)
    }
}

// TODO: load match into str<N> and assert that it's the correct length
// impl<N> From<Match> for str<N>

trait Regex {
    fn find_match(self, input: [u8]) -> Match;
}

// Empty
impl Regex for () {
    fn find_match(_self: Self, input: [u8]) -> Match {
        Match::empty(input)
    }
}

// Exact
impl<let N: u32> Regex for str<N> {
    fn find_match(self, input: [u8]) -> Match {
        let mut leftover = input;
        let mut matches_input = true;
        let self_as_bytes = self.as_bytes();
        for c in self_as_bytes {
            if leftover.len() != 0 {
                let (first_elem, popped_slice) = leftover.pop_front();
                leftover = popped_slice;
                matches_input &= first_elem == c;
            } else {
                matches_input = false;
            }
        }
        if matches_input {
            Match { succeeded: true, match_ends: self_as_bytes.len(), leftover }
        } else {
            Match { succeeded: false, match_ends: 0, leftover: input }
        }
    }
}

// And
impl<T, U> Regex for (T, U)
where
    T: Regex,
    U: Regex,
{
    fn find_match(self, input: [u8]) -> Match {
        let lhs_result = self.0.find_match(input);
        if lhs_result.succeeded {
            let rhs_result = self.1.find_match(lhs_result.leftover);
            if rhs_result.succeeded {
                Match {
                    succeeded: true,
                    match_ends: lhs_result.match_ends + rhs_result.match_ends,
                    leftover: rhs_result.leftover,
                }
            } else {
                Match { succeeded: false, match_ends: 0, leftover: input }
            }
        } else {
            Match { succeeded: false, match_ends: 0, leftover: input }
        }
    }
}

// N T's: (T, (T, (T, T)))
struct Repeated<T, let N: u32> {
    inner: T,
}

impl<T, let N: u32> Regex for Repeated<T, N>
where
    T: Regex,
{
    fn find_match(self, input: [u8]) -> Match {
        let mut result = Match::empty(input);
        for _ in 0..N {
            if result.succeeded {
                let next_result = self.inner.find_match(result.leftover);
                result = Match {
                    succeeded: next_result.succeeded,
                    match_ends: result.match_ends + next_result.match_ends,
                    leftover: next_result.leftover,
                };
            }
        }
        result
    }
}

struct Or<T, U> {
    lhs: T,
    rhs: U,
}

impl<T, U> Regex for Or<T, U>
where
    T: Regex,
    U: Regex,
{
    fn find_match(self, input: [u8]) -> Match {
        let lhs_result = self.lhs.find_match(input);
        if lhs_result.succeeded {
            lhs_result
        } else {
            self.rhs.find_match(input)
        }
    }
}

struct Question<T> {
    inner: T,
}

impl<T> Regex for Question<T>
where
    T: Regex,
{
    fn find_match(self, input: [u8]) -> Match {
        Or { lhs: self.inner, rhs: () }.find_match(input)
    }
}

// 0 <= num_matches <= N
struct Star<T, let N: u32> {
    inner: T,
}

impl<T, let N: u32> Regex for Star<T, N>
where
    T: Regex,
{
    fn find_match(self, input: [u8]) -> Match {
        let regex: Repeated<_, N> = Repeated { inner: Question { inner: self.inner } };
        regex.find_match(input)
    }
}

// 0 < num_matches <= N
struct Plus<T, let N: u32, let N_PRED: u32> {
    inner: T,
}

impl<T, let N: u32, let N_PRED: u32> Regex for Plus<T, N, N_PRED>
where
    T: Regex,
{
    fn find_match(self, input: [u8]) -> Match {
        std::static_assert(N_PRED + 1 == N, "N - 1 != N_PRED");
        let star: Star<T, N_PRED> = Star { inner: self.inner };
        (self.inner, star).find_match(input)
    }
}

fn main() {
    // gr(a|e)y
    let graey_regex = ("gr", (Or { lhs: "a", rhs: "e" }, "y"));

    // NOTE: leftover ignored in Eq: Match
    let result = graey_regex.find_match("gray".as_bytes().as_slice());
    println(result);
    assert_eq(result, Match { succeeded: true, match_ends: 4, leftover: &[] });

    // NOTE: leftover ignored in Eq: Match
    let result = graey_regex.find_match("grey".as_bytes().as_slice());
    println(result);
    assert_eq(result, Match { succeeded: true, match_ends: 4, leftover: &[] });

    // colou?r
    let colour_regex = ("colo", (Question { inner: "u" }, "r"));

    let result = colour_regex.find_match("color".as_bytes().as_slice());
    println(result);
    assert_eq(result, Match { succeeded: true, match_ends: 5, leftover: &[] });

    let result = colour_regex.find_match("colour".as_bytes().as_slice());
    println(result);
    assert_eq(result, Match { succeeded: true, match_ends: 6, leftover: &[] });

    // parse the empty string three times
    // EMPTY{3}
    let three_empties_regex: Repeated<(), 3> = Repeated { inner: () };

    let result = three_empties_regex.find_match("111".as_bytes().as_slice());
    println(result);
    assert_eq(result, Match { succeeded: true, match_ends: 0, leftover: &[] });

    // 1{0}
    let zero_ones_regex: Repeated<str<1>, 0> = Repeated { inner: "1" };

    let result = zero_ones_regex.find_match("111".as_bytes().as_slice());
    println(result);
    assert_eq(result, Match { succeeded: true, match_ends: 0, leftover: &[] });

    // 1{1}
    let one_ones_regex: Repeated<str<1>, 1> = Repeated { inner: "1" };

    let result = one_ones_regex.find_match("111".as_bytes().as_slice());
    println(result);
    assert_eq(result, Match { succeeded: true, match_ends: 1, leftover: &[] });

    // 1{2}
    let two_ones_regex: Repeated<str<1>, 2> = Repeated { inner: "1" };

    let result = two_ones_regex.find_match("111".as_bytes().as_slice());
    println(result);
    assert_eq(result, Match { succeeded: true, match_ends: 2, leftover: &[] });

    // 1{3}
    let three_ones_regex: Repeated<str<1>, 3> = Repeated { inner: "1" };

    let result = three_ones_regex.find_match("1111".as_bytes().as_slice());
    println(result);
    assert_eq(result, Match { succeeded: true, match_ends: 3, leftover: &[] });
    // TODO(https://github.com/noir-lang/noir/issues/6285): re-enable these cases and complete the test using array_regex below
    //
    // // 1*
    // let ones_regex: Star<str<1>, 5> = Star { inner: "1" };
    //
    // let result = ones_regex.find_match("11000".as_bytes().as_slice());
    // println(result);
    // assert_eq(result, Match { succeeded: true, match_ends: 2, leftover: &[] });
    //
    // let result = ones_regex.find_match("11".as_bytes().as_slice());
    // println(result);
    // assert_eq(result, Match { succeeded: true, match_ends: 2, leftover: &[] });
    //
    // let result = ones_regex.find_match("111111".as_bytes().as_slice());
    // println(result);
    // assert_eq(result, Match { succeeded: true, match_ends: 5, leftover: &[] });
    //
    //
    // // 1+
    // let nonempty_ones_regex: Plus<str<1>, 5, 4> = Plus { inner: "1" };
    //
    // let result = nonempty_ones_regex.find_match("111111".as_bytes().as_slice());
    // println(result);
    // assert_eq(result, Match { succeeded: true, match_ends: 5, leftover: &[] });
    //
    // // 2^n-1 in binary: 1+0
    // let pred_pow_two_regex = (nonempty_ones_regex, "0");
    //
    // let result = pred_pow_two_regex.find_match("1110".as_bytes().as_slice());
    // println(result);
    // assert_eq(result, Match { succeeded: true, match_ends: 3, leftover: &[] });
    //
    // // (0|1)*
    // let binary_regex: Star<Or<str<1>, str<1>>, 5> = Star { inner: Or { lhs: "0", rhs: "1" } };
    //
    // let result = binary_regex.find_match("110100".as_bytes().as_slice());
    // println(result);
    // assert_eq(result, Match { succeeded: true, match_ends: 5, leftover: &[] });
    //
    // // even numbers in binary: 1(0|1)*0
    // let even_binary_regex = ("1", (binary_regex, "0"));
    //
    // let result = even_binary_regex.find_match("1111110".as_bytes().as_slice());
    // println(result);
    // assert_eq(result, Match { succeeded: true, match_ends: 6, leftover: &[] });
    // 2-letter capitalized words: [A-Z][a-z]
    // numbers: \d+
    // [0-9]+
    // words: \w+
    // [a-Z]+
    // adapted URL parser: (https?:\/\/)?([\da-z.\-]+)\.([a-z.]+)([\/\w \.\-]*)*\/?
    // // panics (at compile time) when input string is too short
    // let foo_regex = (
    //     "colo",
    //     (
    //         Question {
    //             inner: "u",
    //         },
    //         "r"
    //     )
    // );
    //
    // let result = foo_regex.find_match("colo".as_bytes().as_slice());
    // println(result);
    // assert_eq(result, Match {
    //     succeeded: true,
    //     match_ends: 4,
    //     leftover: &[],
    // });
}

// TODO
// array_regex execution_success test:
// use to complete test once https://github.com/noir-lang/noir/issues/6285 is resolved
//
// // offset <= len <= N
// struct Bvec<T, let N: u32> {
//     inner: [T; N],
//
//     // elements at indices < offset are zero
//     offset: u32,
//
//     // elements at indices >= len are zero
//     len: u32,
// }
//
// impl<T, let N: u32> Eq for Bvec<T, N> where T: Eq {
//     fn eq(self, other: Self) -> bool {
//         (self.inner == other.inner) &
//             (self.offset == other.offset) &
//             (self.len == other.len)
//     }
// }
//
// impl<T, let N: u32> Bvec<T, N> {
//     fn empty() -> Self {
//         Self { inner: [std::mem::zeroed(); N], offset: 0, len: 0 }
//     }
//
//     fn new(array: [T; N]) -> Self {
//         let mut result = Bvec::empty();
//         for x in array {
//             result = result.push(x);
//         }
//         result
//     }
//
//     // pushing when len == N is a no-op
//     fn push(self, x: T) -> Self {
//         let mut inner = self.inner;
//         let mut len = self.len;
//         if self.len < N {
//             inner[self.len] = x;
//             len += 1;
//         }
//
//         Self { inner, offset: self.offset, len }
//     }
//
//     fn pop_front(self) -> (T, Self) {
//         assert(self.offset <= self.inner.len());
//         assert(self.len != 0);
//
//         let first_elem = self.inner[self.offset];
//         let popped_slice = Self { inner: self.inner, offset: self.offset + 1, len: self.len - 1 };
//
//         (first_elem, popped_slice)
//     }
// }
//
// struct Match<let N: u32> {
//     succeeded: bool,
//     match_ends: u32,
//     leftover: Bvec<u8, N>,
// }
//
// impl<let N: u32> Match<N> {
//     fn empty(leftover: Bvec<u8, N>) -> Self {
//         Match { succeeded: true, match_ends: 0, leftover }
//     }
//
//     fn failed(leftover: Bvec<u8, N>) -> Self {
//         Match { succeeded: false, match_ends: 0, leftover }
//     }
// }
//
// impl<let N: u32> Eq for Match<N> {
//     fn eq(self, other: Self) -> bool {
//         (self.succeeded == other.succeeded) &
//             (self.match_ends == other.match_ends) &
//             (self.leftover == other.leftover)
//     }
// }
//
// // TODO: load match into str<N> and assert that it's the correct length
// // impl<N> From<Match> for str<N>
//
// trait Regex {
//     // Perform a match without backtracking
//     fn find_match<let N: u32>(self, input: Bvec<u8, N>) -> Match<N>;
// }
//
// // Empty
// impl Regex for () {
//     fn find_match<let N: u32>(_self: Self, input: Bvec<u8, N>) -> Match<N> {
//         Match::empty(input)
//     }
// }
//
// // Exact
// impl<let M: u32> Regex for str<M> {
//     fn find_match<let N: u32>(self, input: Bvec<u8, N>) -> Match<N> {
//         let mut leftover = input;
//         let mut matches_input = true;
//         let self_as_bytes = self.as_bytes();
//         for c in self_as_bytes {
//             if leftover.len != 0 {
//                 let (first_elem, popped_slice) = leftover.pop_front();
//                 leftover = popped_slice;
//                 matches_input &= first_elem == c;
//             } else {
//                 matches_input = false;
//             }
//         }
//         if matches_input {
//             Match {
//                 succeeded: true,
//                 match_ends: self_as_bytes.len(),
//                 leftover,
//             }
//         } else {
//             Match {
//                 succeeded: false,
//                 match_ends: 0,
//                 leftover: input,
//             }
//         }
//     }
// }
//
// // And
// impl<T, U> Regex for (T, U) where T: Regex, U: Regex {
//     fn find_match<let N: u32>(self, input: Bvec<u8, N>) -> Match<N> {
//         let lhs_result = self.0.find_match(input);
//         if lhs_result.succeeded {
//             let rhs_result = self.1.find_match(lhs_result.leftover);
//             if rhs_result.succeeded {
//                 Match {
//                     succeeded: true,
//                     match_ends: lhs_result.match_ends + rhs_result.match_ends,
//                     leftover: rhs_result.leftover,
//                 }
//             } else {
//                 Match {
//                     succeeded: false,
//                     match_ends: 0,
//                     leftover: input,
//                 }
//             }
//         } else {
//             Match {
//                 succeeded: false,
//                 match_ends: 0,
//                 leftover: input,
//             }
//         }
//     }
// }
//
// // N T's: (T, (T, (T, T)))
// struct Repeated<T, let N: u32> {
//     inner: T,
// }
//
// impl<T, let M: u32> Regex for Repeated<T, M> where T: Regex {
//     fn find_match<let N: u32>(self, input: Bvec<u8, N>) -> Match<N> {
//         let mut result = Match::empty(input);
//         for _ in 0..M {
//             if result.succeeded {
//                 let next_result = self.inner.find_match(result.leftover);
//                 result = Match {
//                     succeeded: next_result.succeeded,
//                     match_ends: result.match_ends + next_result.match_ends,
//                     leftover: next_result.leftover,
//                 };
//             }
//         }
//         result
//     }
// }
//
// struct Or<T, U> {
//     lhs: T,
//     rhs: U,
// }
//
// impl<T, U> Regex for Or<T, U> where T: Regex, U: Regex {
//     fn find_match<let N: u32>(self, input: Bvec<u8, N>) -> Match<N> {
//         let lhs_result = self.lhs.find_match(input);
//         if lhs_result.succeeded {
//             lhs_result
//         } else {
//             self.rhs.find_match(input)
//         }
//     }
// }
//
// struct Question<T> {
//     inner: T,
// }
//
// impl<T> Regex for Question<T> where T: Regex {
//     fn find_match<let N: u32>(self, input: Bvec<u8, N>) -> Match<N> {
//         Or {
//             lhs: self.inner,
//             rhs: (),
//         }.find_match(input)
//     }
// }
//
// // 0 <= num_matches <= N
// struct Star<T, let N: u32> {
//     inner: T,
// }
//
// impl<T, let M: u32> Regex for Star<T, M> where T: Regex {
//     fn find_match<let N: u32>(self, input: Bvec<u8, N>) -> Match<N> {
//         let regex: Repeated<_, M> = Repeated {
//             inner: Question { inner: self.inner },
//         };
//         regex.find_match(input)
//     }
// }
//
// // 0 < num_matches <= N
// struct Plus<T, let N: u32, let N_PRED: u32> {
//     inner: T,
// }
//
// impl<T, let M: u32, let M_PRED: u32> Regex for Plus<T, M, M_PRED> where T: Regex {
//     fn find_match<let N: u32>(self, input: Bvec<u8, N>) -> Match<N> {
//         std::static_assert(M_PRED + 1 == M, "M - 1 != M_PRED");
//         let star: Star<T, M_PRED> = Star { inner: self.inner };
//         (
//             self.inner,
//             star
//         ).find_match(input)
//     }
// }
//
// // Repeated is to (,) as AnyOf is to Or
// struct AnyOf<T, let N: u32> {
//     inner: [T; N],
// }
//
// impl<T, let M: u32> Regex for AnyOf<T, M> where T: Regex {
//     fn find_match<let N: u32>(self, input: Bvec<u8, N>) -> Match<N> {
//         let mut result = Match::failed(input);
//         for i in 0..M {
//             if !result.succeeded {
//                 result = self.inner[i].find_match(result.leftover);
//             }
//         }
//         result
//     }
// }
//
// fn reverse_array<T, let N: u32>(input: [T; N]) -> [T; N] {
//     let mut output = [std::mem::zeroed(); N];
//     for i in 0..N {
//         output[i] = input[N - (i + 1)];
//     }
//     output
// }
//
// fn main() {
//     assert_eq(reverse_array([1, 2, 3, 4]), [4, 3, 2, 1]);
//
//     let mut xs: Bvec<u8, 3> = Bvec::empty();
//
//     xs = xs.push(0);
//     assert_eq(xs, Bvec { inner: [0, 0, 0], offset: 0, len: 1 });
//
//     xs = xs.push(1);
//     assert_eq(xs, Bvec { inner: [0, 1, 0], offset: 0, len: 2 });
//
//     xs = xs.push(2);
//     assert_eq(xs, Bvec { inner: [0, 1, 2], offset: 0, len: 3 });
//
//     xs = xs.push(3);
//     assert_eq(xs, Bvec { inner: [0, 1, 2], offset: 0, len: 3 });
//
//     let ys = Bvec::new([0, 1, 2]);
//     assert_eq(xs, ys);
//
//     // test that pop_front gives all contents, in order,
//     // followed by std::mem::zeroed()
//     println(xs);
//     let (x, new_xs) = xs.pop_front();
//     assert_eq(x, 0);
//
//     xs = new_xs;
//     println(xs);
//     let (x, new_xs) = xs.pop_front();
//     assert_eq(x, 1);
//
//     xs = new_xs;
//     println(xs);
//     let (x, new_xs) = xs.pop_front();
//     assert_eq(x, 2);
//
//     xs = new_xs;
//     println(xs);
//     if xs.len != 0 {
//         let (x, _new_xs) = xs.pop_front();
//         assert_eq(x, std::mem::zeroed());
//     }
//
//     assert_eq(new_xs, Bvec { inner: [0, 1, 2], offset: 3, len: 0 });
//
//     // gr(a|e)y
//     let graey_regex = ("gr", (Or { lhs: "a", rhs: "e" }, "y"));
//
//     let result = graey_regex.find_match(Bvec::new("gray".as_bytes()));
//     println(result);
//     assert(result.succeeded);
//     assert_eq(result.match_ends, 4);
//     assert_eq(result.leftover.len, 0);
//
//     let result = graey_regex.find_match(Bvec::new("grey".as_bytes()));
//     println(result);
//     assert(result.succeeded);
//     assert_eq(result.match_ends, 4);
//     assert_eq(result.leftover.len, 0);
//
//     // colou?r
//     let colour_regex = ("colo", (Question { inner: "u" }, "r"));
//
//     let result = colour_regex.find_match(Bvec::new("color".as_bytes()));
//     println(result);
//     assert(result.succeeded);
//     assert_eq(result.match_ends, 5);
//     assert_eq(result.leftover.len, 0);
//
//     let result = colour_regex.find_match(Bvec::new("colour".as_bytes()));
//     println(result);
//     assert(result.succeeded);
//     assert_eq(result.match_ends, 6);
//     assert_eq(result.leftover.len, 0);
//
//     // parse the empty string three times
//     // EMPTY{3}
//     let three_empties_regex: Repeated<(), 3> = Repeated { inner: () };
//
//     let result = three_empties_regex.find_match(Bvec::new("111".as_bytes()));
//     println(result);
//     assert(result.succeeded);
//     assert_eq(result.match_ends, 0);
//     assert_eq(result.leftover.len, 3);
//
//     // 1{0}
//     let zero_ones_regex: Repeated<str<1>, 0> = Repeated { inner: "1" };
//
//     let result = zero_ones_regex.find_match(Bvec::new("111".as_bytes()));
//     println(result);
//     assert(result.succeeded);
//     assert_eq(result.match_ends, 0);
//     assert_eq(result.leftover.len, 3);
//
//     // 1{1}
//     let one_ones_regex: Repeated<str<1>, 1> = Repeated { inner: "1" };
//
//     let result = one_ones_regex.find_match(Bvec::new("111".as_bytes()));
//     println(result);
//     assert(result.succeeded);
//     assert_eq(result.match_ends, 1);
//     assert_eq(result.leftover.len, 2);
//
//     // 1{2}
//     let two_ones_regex: Repeated<str<1>, 2> = Repeated { inner: "1" };
//
//     let result = two_ones_regex.find_match(Bvec::new("111".as_bytes()));
//     println(result);
//     assert(result.succeeded);
//     assert_eq(result.match_ends, 2);
//     assert_eq(result.leftover.len, 1);
//
//     // 1{3}
//     let three_ones_regex: Repeated<str<1>, 3> = Repeated { inner: "1" };
//
//     let result = three_ones_regex.find_match(Bvec::new("1111".as_bytes()));
//     println(result);
//     assert(result.succeeded);
//     assert_eq(result.match_ends, 3);
//     assert_eq(result.leftover.len, 1);
//
//     // 1*
//     let ones_regex: Star<str<1>, 5> = Star { inner: "1" };
//
//     let result = ones_regex.find_match(Bvec::new("11000".as_bytes()));
//     println(result);
//     assert(result.succeeded);
//     assert_eq(result.match_ends, 2);
//     assert_eq(result.leftover.len, 3);
//
//     let result = ones_regex.find_match(Bvec::new("11".as_bytes()));
//     println(result);
//     assert(result.succeeded);
//     assert_eq(result.match_ends, 2);
//     assert_eq(result.leftover.len, 0);
//
//     let result = ones_regex.find_match(Bvec::new("111111".as_bytes()));
//     println(result);
//     assert(result.succeeded);
//     assert_eq(result.match_ends, 5);
//     assert_eq(result.leftover.len, 1);
//
//     // 1+
//     let nonempty_ones_regex: Plus<str<1>, 5, 4> = Plus { inner: "1" };
//
//     let result = nonempty_ones_regex.find_match(Bvec::new("111111".as_bytes()));
//     println(result);
//     assert(result.succeeded);
//     assert_eq(result.match_ends, 5);
//     assert_eq(result.leftover.len, 1);
//
//     // 2^n-1 in binary: 1+0
//     let pred_pow_two_regex = (nonempty_ones_regex, "0");
//
//     let result = pred_pow_two_regex.find_match(Bvec::new("1110".as_bytes()));
//     println(result);
//     assert(result.succeeded);
//     assert_eq(result.match_ends, 4);
//     assert_eq(result.leftover.len, 0);
//
//     // (0|1)*
//     let binary_regex: Star<Or<str<1>, str<1>>, 5> = Star { inner: Or { lhs: "0", rhs: "1" } };
//
//     let result = binary_regex.find_match(Bvec::new("110100".as_bytes()));
//     println(result);
//     assert(result.succeeded);
//     assert_eq(result.match_ends, 5);
//     assert_eq(result.leftover.len, 1);
//
//     // even numbers in binary: 1(0|1)*0
//     let even_binary_regex = ("1", (binary_regex, "0"));
//
//     let result = even_binary_regex.find_match(Bvec::new("1111110".as_bytes()));
//     println(result);
//     assert(result.succeeded);
//     assert_eq(result.match_ends, 7);
//     assert_eq(result.leftover.len, 0);
//
//     // digit: \d+
//     // [0-9]
//     let digit_regex = AnyOf {
//         inner: [
//             "0",
//             "1",
//             "2",
//             "3",
//             "4",
//             "5",
//             "6",
//             "7",
//             "8",
//             "9"
//         ]
//     };
//
//     let result = digit_regex.find_match(Bvec::new("157196345823795".as_bytes()));
//     println(result);
//     assert(result.succeeded);
//     assert_eq(result.match_ends, 1);
//     assert_eq(result.leftover.len, 14);
//
//     let result = digit_regex.find_match(Bvec::new("hi".as_bytes()));
//     println(result);
//     assert(!result.succeeded);
//     assert_eq(result.match_ends, 0);
//     assert_eq(result.leftover.len, 2);
//
//     // digits: \d+
//     // [0-9]+
//     let digits_regex: Plus<AnyOf<str<1>, 10>, 32, 31> = Plus { inner: digit_regex };
//
//     let result = digits_regex.find_match(Bvec::new("123456789012345".as_bytes()));
//     println(result);
//     assert(result.succeeded);
//     assert_eq(result.match_ends, 15);
//     assert_eq(result.leftover.len, 0);
//
//     let result = digits_regex.find_match(Bvec::new("123456789012345 then words".as_bytes()));
//     println(result);
//     assert(result.succeeded);
//     assert_eq(result.match_ends, 15);
//     assert_eq(result.leftover.len, 11);
//
//     // multiples of 10
//     // apply to a reversed input string (because there isn't backtracking)
//     // 0\d+
//     let backwards_mult_of_10_regex = ("0", digits_regex);
//
//     let result = backwards_mult_of_10_regex.find_match(Bvec::new(reverse_array("1230".as_bytes())));
//     println(result);
//     assert(result.succeeded);
//     assert_eq(result.match_ends, 4);
//     assert_eq(result.leftover.len, 0);
//
//     let ten_pow_16: str<17> = "10000000000000000";
//     let result = backwards_mult_of_10_regex.find_match(Bvec::new(reverse_array(ten_pow_16.as_bytes())));
//     println(result);
//     assert(result.succeeded);
//     assert_eq(result.match_ends, 17);
//     assert_eq(result.leftover.len, 0);
//     // adapted URL parser: (https?:\/\/)?([\da-c.\-]+)\.([a-c.]+)([\/\w \.\-]*)*\/?
// }

