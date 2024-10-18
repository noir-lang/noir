// a more thorough test for trait function/method calls from within trait function/method calls
// This tests all combinations of caller+callee pairs. For each of these, the following combination of properties are tried:
//   *) method (has self parameter) vs function (no self parameter)
//   *) default vs overriden vs overriden (no default)
// test order is:
// 1) trait method -> trait method
// 1a) trait default method -> trait default method
// 1b) trait default method -> trait overriden method
// 1c) trait default method -> trait overriden (no default) method
// 1d) trait overriden method -> trait default method
// 1e) trait overriden method -> trait overriden method
// 1f) trait overriden method -> trait overriden (no default) method
// 1g) trait overriden (no default) method -> trait default method
// 1h) trait overriden (no default) method -> trait overriden method
// 1i) trait overriden (no default) method -> trait overriden (no default) method
// 2) trait method -> trait function
// 2a) - subcases are the same as the above
// ...
// 2i)
// 3{a..i}) trait function -> trait method
// 4{a..i}) trait function -> trait function
// 1) trait method -> trait method
// 1a) trait default method -> trait default method
trait Trait1a {
    fn trait_method1(self) -> Field {
        self.trait_method2() * 7892 - self.vl
    }
    fn trait_method2(self) -> Field {
        let _ = self;
        43278
    }
}
struct Struct1a {
    vl: Field,
}
impl Trait1a for Struct1a {}
// 1b) trait default method -> trait overriden method
trait Trait1b {
    fn trait_method1(self) -> Field {
        self.trait_method2() * 2832 - self.vl
    }
    fn trait_method2(self) -> Field {
        9323
    }
}
struct Struct1b {
    vl: Field,
}
impl Trait1b for Struct1b {
    fn trait_method2(self) -> Field {
        let _ = self;
        2394
    }
}
// 1c) trait default method -> trait overriden (no default) method
trait Trait1c {
    fn trait_method1(self) -> Field {
        self.trait_method2() * 7635 - self.vl
    }
    fn trait_method2(self) -> Field;
}
struct Struct1c {
    vl: Field,
}
impl Trait1c for Struct1c {
    fn trait_method2(self) -> Field {
        let _ = self;
        5485
    }
}
// 1d) trait overriden method -> trait default method
trait Trait1d {
    fn trait_method1(self) -> Field {
        self.trait_method2() * 2825 - self.vl
    }
    fn trait_method2(self) -> Field {
        let _ = self;
        29341
    }
}
struct Struct1d {
    vl: Field,
}
impl Trait1d for Struct1d {
    fn trait_method1(self) -> Field {
        self.trait_method2() * 9342 - self.vl
    }
}
// 1e) trait overriden method -> trait overriden method
trait Trait1e {
    fn trait_method1(self) -> Field {
        self.trait_method2() * 85465 - self.vl
    }
    fn trait_method2(self) -> Field {
        2381
    }
}
struct Struct1e {
    vl: Field,
}
impl Trait1e for Struct1e {
    fn trait_method1(self) -> Field {
        self.trait_method2() * 47324 - self.vl
    }
    fn trait_method2(self) -> Field {
        let _ = self;
        58945
    }
}
// 1f) trait overriden method -> trait overriden (no default) method
trait Trait1f {
    fn trait_method1(self) -> Field {
        self.trait_method2() * 43257 - self.vl
    }
    fn trait_method2(self) -> Field;
}
struct Struct1f {
    vl: Field,
}
impl Trait1f for Struct1f {
    fn trait_method1(self) -> Field {
        self.trait_method2() * 34875 - self.vl
    }
    fn trait_method2(self) -> Field {
        let _ = self;
        5748
    }
}
// 1g) trait overriden (no default) method -> trait default method
trait Trait1g {
    fn trait_method1(self) -> Field;
    fn trait_method2(self) -> Field {
        let _ = self;
        37845
    }
}
struct Struct1g {
    vl: Field,
}
impl Trait1g for Struct1g {
    fn trait_method1(self) -> Field {
        self.trait_method2() * 7854 - self.vl
    }
}
// 1h) trait overriden (no default) method -> trait overriden method
trait Trait1h {
    fn trait_method1(self) -> Field;
    fn trait_method2(self) -> Field {
        7823
    }
}
struct Struct1h {
    vl: Field,
}
impl Trait1h for Struct1h {
    fn trait_method1(self) -> Field {
        self.trait_method2() * 3482 - self.vl
    }
    fn trait_method2(self) -> Field {
        let _ = self;
        8542
    }
}
// 1i) trait overriden (no default) method -> trait overriden (no default) method
trait Trait1i {
    fn trait_method1(self) -> Field;
    fn trait_method2(self) -> Field;
}
struct Struct1i {
    vl: Field,
}
impl Trait1i for Struct1i {
    fn trait_method1(self) -> Field {
        self.trait_method2() * 23478 - self.vl
    }
    fn trait_method2(self) -> Field {
        let _ = self;
        98543
    }
}
// 2) trait method -> trait function
// 2a) trait default method -> trait default function
trait Trait2a {
    fn trait_method1(self) -> Field {
        Self::trait_function2() * 2385 - self.vl
    }
    fn trait_function2() -> Field {
        7843
    }
}
struct Struct2a {
    vl: Field,
}
impl Trait2a for Struct2a {}
// 2b) trait default method -> trait overriden function
trait Trait2b {
    fn trait_method1(self) -> Field {
        Self::trait_function2() * 6583 - self.vl
    }
    fn trait_function2() -> Field {
        3752
    }
}
struct Struct2b {
    vl: Field,
}
impl Trait2b for Struct2b {
    fn trait_function2() -> Field {
        8477
    }
}
// 2c) trait default method -> trait overriden (no default) function
trait Trait2c {
    fn trait_method1(self) -> Field {
        Self::trait_function2() * 2831 - self.vl
    }
    fn trait_function2() -> Field;
}
struct Struct2c {
    vl: Field,
}
impl Trait2c for Struct2c {
    fn trait_function2() -> Field {
        8342
    }
}
// 2d) trait overriden method -> trait default function
trait Trait2d {
    fn trait_method1(self) -> Field {
        Self::trait_function2() * 924 - self.vl
    }
    fn trait_function2() -> Field {
        384
    }
}
struct Struct2d {
    vl: Field,
}
impl Trait2d for Struct2d {
    fn trait_method1(self) -> Field {
        Self::trait_function2() * 3984 - self.vl
    }
}
// 2e) trait overriden method -> trait overriden function
trait Trait2e {
    fn trait_method1(self) -> Field {
        Self::trait_function2() * 3642 - self.vl
    }
    fn trait_function2() -> Field {
        97342
    }
}
struct Struct2e {
    vl: Field,
}
impl Trait2e for Struct2e {
    fn trait_method1(self) -> Field {
        Self::trait_function2() * 7363 - self.vl
    }
    fn trait_function2() -> Field {
        39400
    }
}
// 2f) trait overriden method -> trait overriden (no default) function
trait Trait2f {
    fn trait_method1(self) -> Field {
        Self::trait_function2() * 2783 - self.vl
    }
    fn trait_function2() -> Field;
}
struct Struct2f {
    vl: Field,
}
impl Trait2f for Struct2f {
    fn trait_method1(self) -> Field {
        Self::trait_function2() * 6362 - self.vl
    }
    fn trait_function2() -> Field {
        72311
    }
}
// 2g) trait overriden (no default) method -> trait default function
trait Trait2g {
    fn trait_method1(self) -> Field;
    fn trait_function2() -> Field {
        19273
    }
}
struct Struct2g {
    vl: Field,
}
impl Trait2g for Struct2g {
    fn trait_method1(self) -> Field {
        Self::trait_function2() * 9123 - self.vl
    }
}
// 2h) trait overriden (no default) method -> trait overriden function
trait Trait2h {
    fn trait_method1(self) -> Field;
    fn trait_function2() -> Field {
        1281
    }
}
struct Struct2h {
    vl: Field,
}
impl Trait2h for Struct2h {
    fn trait_method1(self) -> Field {
        Self::trait_function2() * 4833 - self.vl
    }
    fn trait_function2() -> Field {
        5335
    }
}
// 2i) trait overriden (no default) method -> trait overriden (no default) function
trait Trait2i {
    fn trait_method1(self) -> Field;
    fn trait_function2() -> Field;
}
struct Struct2i {
    vl: Field,
}
impl Trait2i for Struct2i {
    fn trait_method1(self) -> Field {
        Self::trait_function2() * 2291 - self.vl
    }
    fn trait_function2() -> Field {
        3322
    }
}
// 3 trait function -> trait method
// 3a) trait default function -> trait default method
trait Trait3a {
    fn trait_function1(a: Field, b: Self) -> Field {
        b.trait_method2() * 8344 - b.vl + a
    }
    fn trait_method2(self) -> Field {
        let _ = self;
        19212
    }
}
struct Struct3a {
    vl: Field,
}
impl Trait3a for Struct3a {}
// 3b) trait default function -> trait overriden method
trait Trait3b {
    fn trait_function1(a: Field, b: Self) -> Field {
        b.trait_method2() * 9233 - b.vl + a
    }
    fn trait_method2(self) -> Field {
        9111
    }
}
struct Struct3b {
    vl: Field,
}
impl Trait3b for Struct3b {
    fn trait_method2(self) -> Field {
        let _ = self;
        2392
    }
}
// 3c) trait default function -> trait overriden (no default) method
trait Trait3c {
    fn trait_function1(a: Field, b: Self) -> Field {
        b.trait_method2() * 2822 - b.vl + a
    }
    fn trait_method2(self) -> Field;
}
struct Struct3c {
    vl: Field,
}
impl Trait3c for Struct3c {
    fn trait_method2(self) -> Field {
        let _ = self;
        7743
    }
}
// 3d) trait overriden function -> trait default method
trait Trait3d {
    fn trait_function1(a: Field, b: Self) -> Field {
        b.trait_method2() * 291 - b.vl + a
    }
    fn trait_method2(self) -> Field {
        let _ = self;
        3328
    }
}
struct Struct3d {
    vl: Field,
}
impl Trait3d for Struct3d {
    fn trait_function1(a: Field, b: Self) -> Field {
        b.trait_method2() * 4933 - b.vl + a
    }
}
// 3e) trait overriden function -> trait overriden method
trait Trait3e {
    fn trait_function1(a: Field, b: Self) -> Field {
        b.trait_method2() * 71231 - b.vl + a
    }
    fn trait_method2(self) -> Field {
        373
    }
}
struct Struct3e {
    vl: Field,
}
impl Trait3e for Struct3e {
    fn trait_function1(a: Field, b: Self) -> Field {
        b.trait_method2() * 81232 - b.vl + a
    }
    fn trait_method2(self) -> Field {
        let _ = self;
        80002
    }
}
// 3f) trait overriden function -> trait overriden (no default) method
trait Trait3f {
    fn trait_function1(a: Field, b: Self) -> Field {
        b.trait_method2() * 28223 - b.vl + a
    }
    fn trait_method2(self) -> Field;
}
struct Struct3f {
    vl: Field,
}
impl Trait3f for Struct3f {
    fn trait_function1(a: Field, b: Self) -> Field {
        b.trait_method2() * 29223 - b.vl + a
    }
    fn trait_method2(self) -> Field {
        let _ = self;
        63532
    }
}
// 3g) trait overriden (no default) function -> trait default method
trait Trait3g {
    fn trait_function1(a: Field, b: Self) -> Field;
    fn trait_method2(self) -> Field {
        let _ = self;
        8887
    }
}
struct Struct3g {
    vl: Field,
}
impl Trait3g for Struct3g {
    fn trait_function1(a: Field, b: Self) -> Field {
        b.trait_method2() * 31337 - b.vl + a
    }
}
// 3h) trait overriden (no default) function -> trait overriden method
trait Trait3h {
    fn trait_function1(a: Field, b: Self) -> Field;
    fn trait_method2(self) -> Field {
        293
    }
}
struct Struct3h {
    vl: Field,
}
impl Trait3h for Struct3h {
    fn trait_function1(a: Field, b: Self) -> Field {
        b.trait_method2() * 74747 - b.vl + a
    }
    fn trait_method2(self) -> Field {
        let _ = self;
        6283
    }
}
// 3i) trait overriden (no default) function -> trait overriden (no default) method
trait Trait3i {
    fn trait_function1(a: Field, b: Self) -> Field;
    fn trait_method2(self) -> Field;
}
struct Struct3i {
    vl: Field,
}
impl Trait3i for Struct3i {
    fn trait_function1(a: Field, b: Self) -> Field {
        b.trait_method2() * 1237 - b.vl + a
    }
    fn trait_method2(self) -> Field {
        let _ = self;
        84352
    }
}
// 4) trait function -> trait function
// 4a) trait default function -> trait default function
trait Trait4a {
    fn trait_function1() -> Field {
        Self::trait_function2() * 3842
    }
    fn trait_function2() -> Field {
        2932
    }
}
pub struct Struct4a {
    vl: Field,
}
impl Trait4a for Struct4a {}
// 4b) trait default function -> trait overriden function
trait Trait4b {
    fn trait_function1() -> Field {
        Self::trait_function2() * 3842
    }
    fn trait_function2() -> Field {
        2932
    }
}
pub struct Struct4b {
    vl: Field,
}
impl Trait4b for Struct4b {
    fn trait_function2() -> Field {
        9353
    }
}
// 4c) trait default function -> trait overriden (no default) function
trait Trait4c {
    fn trait_function1() -> Field {
        Self::trait_function2() * 7832
    }
    fn trait_function2() -> Field;
}
pub struct Struct4c {
    vl: Field,
}
impl Trait4c for Struct4c {
    fn trait_function2() -> Field {
        2928
    }
}
// 4d) trait overriden function -> trait default function
trait Trait4d {
    fn trait_function1() -> Field {
        Self::trait_function2() * 2283
    }
    fn trait_function2() -> Field {
        9332
    }
}
pub struct Struct4d {
    vl: Field,
}
impl Trait4d for Struct4d {
    fn trait_function1() -> Field {
        Self::trait_function2() * 8374
    }
}
// 4e) trait overriden function -> trait overriden function
trait Trait4e {
    fn trait_function1() -> Field {
        Self::trait_function2() * 94329
    }
    fn trait_function2() -> Field {
        28328
    }
}
pub struct Struct4e {
    vl: Field,
}
impl Trait4e for Struct4e {
    fn trait_function1() -> Field {
        Self::trait_function2() * 12323
    }
    fn trait_function2() -> Field {
        38434
    }
}
// 4f) trait overriden function -> trait overriden (no default) function
trait Trait4f {
    fn trait_function1() -> Field {
        Self::trait_function2() * 23723
    }
    fn trait_function2() -> Field;
}
pub struct Struct4f {
    vl: Field,
}
impl Trait4f for Struct4f {
    fn trait_function1() -> Field {
        Self::trait_function2() * 21392
    }
    fn trait_function2() -> Field {
        4394
    }
}
// 4g) trait overriden (no default) function -> trait default function
trait Trait4g {
    fn trait_function1() -> Field;
    fn trait_function2() -> Field {
        2932
    }
}
pub struct Struct4g {
    vl: Field,
}
impl Trait4g for Struct4g {
    fn trait_function1() -> Field {
        Self::trait_function2() * 3345
    }
}
// 4h) trait overriden (no default) function -> trait overriden function
trait Trait4h {
    fn trait_function1() -> Field;
    fn trait_function2() -> Field {
        5756
    }
}
pub struct Struct4h {
    vl: Field,
}
impl Trait4h for Struct4h {
    fn trait_function1() -> Field {
        Self::trait_function2() * 6478
    }
    fn trait_function2() -> Field {
        5435
    }
}
// 4i) trait overriden (no default) function -> trait overriden (no default) function
trait Trait4i {
    fn trait_function1() -> Field;
    fn trait_function2() -> Field;
}
pub struct Struct4i {
    vl: Field,
}
impl Trait4i for Struct4i {
    fn trait_function1() -> Field {
        Self::trait_function2() * 8239
    }
    fn trait_function2() -> Field {
        2032
    }
}

fn main() {
    let t1a = Struct1a { vl: 1234 };
    assert(t1a.trait_method1() == 341548742);
    let t1b = Struct1b { vl: 4444 };
    assert(t1b.trait_method1() == 6775364);
    let t1c = Struct1c { vl: 3946 };
    assert(t1c.trait_method1() == 41874029);
    let t1d = Struct1d { vl: 9234 };
    assert(t1d.trait_method1() == 274094388);
    let t1e = Struct1e { vl: 5438 };
    assert(t1e.trait_method1() == 2789507742);
    let t1f = Struct1f { vl: 6237 };
    assert(t1f.trait_method1() == 200455263);
    let t1g = Struct1g { vl: 43587 };
    assert(t1g.trait_method1() == 297191043);
    let t1h = Struct1h { vl: 3984 };
    assert(t1h.trait_method1() == 29739260);
    let t1i = Struct1i { vl: 9234 };
    assert(t1i.trait_method1() == 2313583320);
    let t2a = Struct2a { vl: 4362 };
    assert(t2a.trait_method1() == 18701193);
    let t2b = Struct2b { vl: 8347 };
    assert(t2b.trait_method1() == 55795744);
    let t2c = Struct2c { vl: 1923 };
    assert(t2c.trait_method1() == 23614279);
    let t2d = Struct2d { vl: 92384 };
    assert(t2d.trait_method1() == 1437472);
    let t2e = Struct2e { vl: 83943 };
    assert(t2e.trait_method1() == 290018257);
    let t2f = Struct2f { vl: 8237 };
    assert(t2f.trait_method1() == 460034345);
    let t2g = Struct2g { vl: 1232 };
    assert(t2g.trait_method1() == 175826347);
    let t2h = Struct2h { vl: 7222 };
    assert(t2h.trait_method1() == 25776833);
    let t2i = Struct2i { vl: 1821 };
    assert(t2i.trait_method1() == 7608881);
    let t3a = Struct3a { vl: 93248 };
    assert(Struct3a::trait_function1(5, t3a) == 160211685);
    let t3b = Struct3b { vl: 76763 };
    assert(Struct3b::trait_function1(62, t3b) == 22008635);
    let t3c = Struct3c { vl: 3833 };
    assert(Struct3c::trait_function1(25, t3c) == 21846938);
    let t3d = Struct3d { vl: 5645 };
    assert(Struct3d::trait_function1(73, t3d) == 16411452);
    let t3e = Struct3e { vl: 22912 };
    assert(Struct3e::trait_function1(92, t3e) == 6498699644);
    let t3f = Struct3f { vl: 3256 };
    assert(Struct3f::trait_function1(77, t3f) == 1856592457);
    let t3g = Struct3g { vl: 22832 };
    assert(Struct3g::trait_function1(23, t3g) == 278469110);
    let t3h = Struct3h { vl: 4933 };
    assert(Struct3h::trait_function1(17, t3h) == 469630485);
    let t3i = Struct3i { vl: 39432 };
    assert(Struct3i::trait_function1(54, t3i) == 104304046);
    assert(Struct4a::trait_function1() == 11264744);
    assert(Struct4b::trait_function1() == 35934226);
    assert(Struct4c::trait_function1() == 22932096);
    assert(Struct4d::trait_function1() == 78146168);
    assert(Struct4e::trait_function1() == 473622182);
    assert(Struct4f::trait_function1() == 93996448);
    assert(Struct4g::trait_function1() == 9807540);
    assert(Struct4h::trait_function1() == 35207930);
    assert(Struct4i::trait_function1() == 16741648);
}
