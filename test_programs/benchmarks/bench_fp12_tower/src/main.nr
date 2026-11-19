// Arithmetic in a degree-12 extension tower over a small prime field, shaped like the
// pairing-friendly towers used by BN254/BLS12 verifiers:
//   Fp   = integers mod P = 2^31 - 1
//   Fp2  = Fp[u] / (u^2 + 1)
//   Fp6  = Fp2[v] / (v^3 - (1 + u))
//   Fp12 = Fp6[w] / (w^2 - v)
// Values are updated in place through `&mut self` methods on `let mut` temporaries, and every
// base-field operation ends in a conditional subtraction on a witness value, so after inlining
// `main` is a long chain of blocks with many live references.

global P: u32 = 0x7fffffff;

struct Fp {
    v: u32,
}

impl Fp {
    fn add_assign(&mut self, other: Fp) {
        self.v += other.v;
        if self.v >= P {
            self.v -= P;
        }
    }

    fn neg(self) -> Fp {
        let mut r = Fp { v: P - self.v };
        if r.v == P {
            r.v = 0;
        }
        r
    }

    fn sub_assign(&mut self, other: Fp) {
        self.add_assign(other.neg());
    }

    fn mul_assign(&mut self, other: Fp) {
        // 2^31 = 1 (mod P): fold the high bits of the 62-bit product onto the low bits.
        let x = (self.v as u64) * (other.v as u64);
        let mut r = ((x & (P as u64)) + (x >> 31)) as u32;
        if r >= P {
            r -= P;
        }
        self.v = r;
    }
}

struct Fp2 {
    c0: Fp,
    c1: Fp,
}

impl Fp2 {
    fn add_assign(&mut self, o: Fp2) {
        self.c0.add_assign(o.c0);
        self.c1.add_assign(o.c1);
    }

    fn sub_assign(&mut self, o: Fp2) {
        self.c0.sub_assign(o.c0);
        self.c1.sub_assign(o.c1);
    }

    fn mul_assign(&mut self, o: Fp2) {
        // Karatsuba: three base-field multiplications.
        let mut t0 = self.c0;
        t0.mul_assign(o.c0);
        let mut t1 = self.c1;
        t1.mul_assign(o.c1);
        let mut t2 = self.c0;
        t2.add_assign(self.c1);
        let mut s = o.c0;
        s.add_assign(o.c1);
        t2.mul_assign(s);
        t2.sub_assign(t0);
        t2.sub_assign(t1);
        t0.sub_assign(t1);
        self.c0 = t0;
        self.c1 = t2;
    }

    // Multiplication by the Fp6 non-residue 1 + u.
    fn mul_by_nonresidue(&mut self) {
        let mut c0 = self.c0;
        c0.sub_assign(self.c1);
        self.c1.add_assign(self.c0);
        self.c0 = c0;
    }
}

struct Fp6 {
    b0: Fp2,
    b1: Fp2,
    b2: Fp2,
}

// Karatsuba cross term: (a + b)(c + d) - ac - bd.
fn cross(a: Fp2, b: Fp2, c: Fp2, d: Fp2, ac: Fp2, bd: Fp2) -> Fp2 {
    let mut l = a;
    l.add_assign(b);
    let mut r = c;
    r.add_assign(d);
    l.mul_assign(r);
    l.sub_assign(ac);
    l.sub_assign(bd);
    l
}

impl Fp6 {
    fn add_assign(&mut self, o: Fp6) {
        self.b0.add_assign(o.b0);
        self.b1.add_assign(o.b1);
        self.b2.add_assign(o.b2);
    }

    fn sub_assign(&mut self, o: Fp6) {
        self.b0.sub_assign(o.b0);
        self.b1.sub_assign(o.b1);
        self.b2.sub_assign(o.b2);
    }

    fn mul_assign(&mut self, o: Fp6) {
        // Karatsuba over Fp2: six Fp2 multiplications.
        let mut t0 = self.b0;
        t0.mul_assign(o.b0);
        let mut t1 = self.b1;
        t1.mul_assign(o.b1);
        let mut t2 = self.b2;
        t2.mul_assign(o.b2);
        let mut c0 = cross(self.b1, self.b2, o.b1, o.b2, t1, t2);
        c0.mul_by_nonresidue();
        c0.add_assign(t0);
        let mut c1 = cross(self.b0, self.b1, o.b0, o.b1, t0, t1);
        let mut t2_nr = t2;
        t2_nr.mul_by_nonresidue();
        c1.add_assign(t2_nr);
        let mut c2 = cross(self.b0, self.b2, o.b0, o.b2, t0, t2);
        c2.add_assign(t1);
        self.b0 = c0;
        self.b1 = c1;
        self.b2 = c2;
    }

    // Multiplication by v, the Fp12 non-residue.
    fn mul_by_v(&mut self) {
        let mut b0 = self.b2;
        b0.mul_by_nonresidue();
        self.b2 = self.b1;
        self.b1 = self.b0;
        self.b0 = b0;
    }
}

struct Fp12 {
    c0: Fp6,
    c1: Fp6,
}

impl Fp12 {
    fn mul_assign(&mut self, o: Fp12) {
        // Karatsuba over Fp6: three Fp6 multiplications.
        let mut aa = self.c0;
        aa.mul_assign(o.c0);
        let mut bb = self.c1;
        bb.mul_assign(o.c1);
        let mut c1 = self.c0;
        c1.add_assign(self.c1);
        let mut s = o.c0;
        s.add_assign(o.c1);
        c1.mul_assign(s);
        c1.sub_assign(aa);
        c1.sub_assign(bb);
        bb.mul_by_v();
        aa.add_assign(bb);
        self.c0 = aa;
        self.c1 = c1;
    }
}

fn fp(x: u32) -> Fp {
    assert(x < P);
    Fp { v: x }
}

fn fp12(x: [u32; 12]) -> Fp12 {
    let f2 = |i: u32| Fp2 { c0: fp(x[i]), c1: fp(x[i + 1]) };
    Fp12 {
        c0: Fp6 { b0: f2(0), b1: f2(2), b2: f2(4) },
        c1: Fp6 { b0: f2(6), b1: f2(8), b2: f2(10) },
    }
}

fn coeffs(x: Fp12) -> [u32; 12] {
    let mut out = [0; 12];
    let six = [x.c0, x.c1];
    for i in 0..2 {
        let two = [six[i].b0, six[i].b1, six[i].b2];
        for j in 0..3 {
            out[6 * i + 2 * j] = two[j].c0.v;
            out[6 * i + 2 * j + 1] = two[j].c1.v;
        }
    }
    out
}

fn main(a: [u32; 12], b: [u32; 12]) -> pub [u32; 12] {
    let x = fp12(a);
    let y = fp12(b);
    // Straight-line on purpose: mem2reg runs before loop unrolling, so a loop body would only
    // be analysed once.
    let mut acc = x;
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    acc.mul_assign(y);
    coeffs(acc)
}
