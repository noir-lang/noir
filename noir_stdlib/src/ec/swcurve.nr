pub mod affine {
    // Affine representation of Short Weierstrass curves
    // Points are represented by two-dimensional Cartesian coordinates.
    // Group operations are implemented in terms of those in CurveGroup (in this case, extended Twisted Edwards) coordinates
    // for reasons of efficiency, cf. <https://en.wikibooks.org/wiki/Cryptography/Prime_Curve/Jacobian_Coordinates>.
    use crate::cmp::Eq;
    use crate::ec::is_square;
    use crate::ec::safe_inverse;
    use crate::ec::sqrt;
    use crate::ec::swcurve::curvegroup;

    // Curve specification
    pub struct Curve { // Short Weierstrass curve
        // Coefficients in defining equation y^2 = x^3 + ax + b
        pub a: Field,
        pub b: Field,
        // Generator as point in Cartesian coordinates
        pub gen: Point,
    }
    // Point in Cartesian coordinates
    pub struct Point {
        pub x: Field,
        pub y: Field,
        pub infty: bool, // Indicator for point at infinity
    }

    impl Point {
        // Point constructor
        pub fn new(x: Field, y: Field) -> Self {
            Self { x, y, infty: false }
        }

        // Check if zero
        pub fn is_zero(self) -> bool {
            self.eq(Point::zero())
        }

        // Conversion to CurveGroup coordinates
        pub fn into_group(self) -> curvegroup::Point {
            let Self { x, y, infty } = self;

            if infty {
                curvegroup::Point::zero()
            } else {
                curvegroup::Point::new(x, y, 1)
            }
        }

        // Additive identity
        pub fn zero() -> Self {
            Self { x: 0, y: 0, infty: true }
        }

        // Negation
        pub fn negate(self) -> Self {
            let Self { x, y, infty } = self;
            Self { x, y: 0 - y, infty }
        }
    }

    impl Eq for Point {
        fn eq(self, p: Self) -> bool {
            let Self { x: x1, y: y1, infty: inf1 } = self;
            let Self { x: x2, y: y2, infty: inf2 } = p;

            (inf1 & inf2) | (!inf1 & !inf2 & (x1 == x2) & (y1 == y2))
        }
    }

    impl Curve {
        // Curve constructor
        pub fn new(a: Field, b: Field, gen: Point) -> Curve {
            // Check curve coefficients
            assert(4 * a * a * a + 27 * b * b != 0);

            let curve = Curve { a, b, gen };

            // gen should be on the curve
            assert(curve.contains(curve.gen));

            curve
        }

        // Conversion to CurveGroup coordinates
        pub fn into_group(self) -> curvegroup::Curve {
            let Curve { a, b, gen } = self;

            curvegroup::Curve { a, b, gen: gen.into_group() }
        }

        // Membership check
        pub fn contains(self, p: Point) -> bool {
            let Point { x, y, infty } = p;
            infty | (y * y == x * x * x + self.a * x + self.b)
        }

        // Point addition, implemented in terms of mixed addition for reasons of efficiency
        pub fn add(self, p1: Point, p2: Point) -> Point {
            self.mixed_add(p1, p2.into_group()).into_affine()
        }

        // Mixed point addition, i.e. first argument in affine, second in CurveGroup coordinates.
        pub fn mixed_add(self, p1: Point, p2: curvegroup::Point) -> curvegroup::Point {
            if p1.is_zero() {
                p2
            } else if p2.is_zero() {
                p1.into_group()
            } else {
                let Point { x: x1, y: y1, infty: _inf } = p1;
                let curvegroup::Point { x: x2, y: y2, z: z2 } = p2;
                let you1 = x1 * z2 * z2;
                let you2 = x2;
                let s1 = y1 * z2 * z2 * z2;
                let s2 = y2;

                if you1 == you2 {
                    if s1 != s2 {
                        curvegroup::Point::zero()
                    } else {
                        self.into_group().double(p2)
                    }
                } else {
                    let h = you2 - you1;
                    let r = s2 - s1;
                    let x3 = r * r - h * h * h - 2 * you1 * h * h;
                    let y3 = r * (you1 * h * h - x3) - s1 * h * h * h;
                    let z3 = h * z2;

                    curvegroup::Point::new(x3, y3, z3)
                }
            }
        }

        // Scalar multiplication with scalar represented by a bit array (little-endian convention).
        // If k is the natural number represented by `bits`, then this computes p + ... + p k times.
        pub fn bit_mul<let N: u32>(self, bits: [u1; N], p: Point) -> Point {
            self.into_group().bit_mul(bits, p.into_group()).into_affine()
        }

        // Scalar multiplication (p + ... + p n times)
        pub fn mul(self, n: Field, p: Point) -> Point {
            self.into_group().mul(n, p.into_group()).into_affine()
        }

        // Multi-scalar multiplication (n[0]*p[0] + ... + n[N]*p[N], where * denotes scalar multiplication)
        pub fn msm<let N: u32>(self, n: [Field; N], p: [Point; N]) -> Point {
            let mut out = Point::zero();

            for i in 0..N {
                out = self.add(out, self.mul(n[i], p[i]));
            }

            out
        }

        // Point subtraction
        pub fn subtract(self, p1: Point, p2: Point) -> Point {
            self.add(p1, p2.negate())
        }

        // Simplified Shallue-van de Woestijne-Ulas map-to-curve method; see <https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#name-simplified-shallue-van-de-w>.
        // First determine non-square z != -1 in Field s.t. g(x) - z irreducible over Field and g(b/(z*a)) is square,
        // where g(x) = x^3 + a*x + b. swu_map(c,z,.) then maps a Field element to a point on curve c.
        pub fn swu_map(self, z: Field, u: Field) -> Point {
            // Check whether curve is admissible
            assert(self.a * self.b != 0);

            let Curve { a, b, gen: _gen } = self;

            let tv1 = safe_inverse(z * z * u * u * u * u + u * u * z);
            let x1 = if tv1 == 0 {
                b / (z * a)
            } else {
                (0 - b / a) * (1 + tv1)
            };
            let gx1 = x1 * x1 * x1 + a * x1 + b;
            let x2 = z * u * u * x1;
            let gx2 = x2 * x2 * x2 + a * x2 + b;
            let (x, y) = if is_square(gx1) {
                (x1, sqrt(gx1))
            } else {
                (x2, sqrt(gx2))
            };
            Point::new(x, if u.sgn0() != y.sgn0() { 0 - y } else { y })
        }
    }
}

pub mod curvegroup {
    // CurveGroup representation of Weierstrass curves
    // Points are represented by three-dimensional Jacobian coordinates.
    // See <https://en.wikibooks.org/wiki/Cryptography/Prime_Curve/Jacobian_Coordinates> for details.
    use crate::cmp::Eq;
    use crate::ec::swcurve::affine;

    // Curve specification
    pub struct Curve { // Short Weierstrass curve
        // Coefficients in defining equation y^2 = x^3 + axz^4 + bz^6
        pub a: Field,
        pub b: Field,
        // Generator as point in Cartesian coordinates
        pub gen: Point,
    }
    // Point in three-dimensional Jacobian coordinates
    pub struct Point {
        pub x: Field,
        pub y: Field,
        pub z: Field, // z = 0 corresponds to point at infinity.
    }

    impl Point {
        // Point constructor
        pub fn new(x: Field, y: Field, z: Field) -> Self {
            Self { x, y, z }
        }

        // Check if zero
        pub fn is_zero(self) -> bool {
            self.eq(Point::zero())
        }

        // Conversion to affine coordinates
        pub fn into_affine(self) -> affine::Point {
            let Self { x, y, z } = self;

            if z == 0 {
                affine::Point::zero()
            } else {
                affine::Point::new(x / (z * z), y / (z * z * z))
            }
        }

        // Additive identity
        pub fn zero() -> Self {
            Self { x: 0, y: 0, z: 0 }
        }

        // Negation
        pub fn negate(self) -> Self {
            let Self { x, y, z } = self;
            Self { x, y: 0 - y, z }
        }
    }

    impl Eq for Point {
        fn eq(self, p: Self) -> bool {
            let Self { x: x1, y: y1, z: z1 } = self;
            let Self { x: x2, y: y2, z: z2 } = p;

            ((z1 == 0) & (z2 == 0))
                | (
                    (z1 != 0)
                        & (z2 != 0)
                        & (x1 * z2 * z2 == x2 * z1 * z1)
                        & (y1 * z2 * z2 * z2 == y2 * z1 * z1 * z1)
                )
        }
    }

    impl Curve {
        // Curve constructor
        pub fn new(a: Field, b: Field, gen: Point) -> Curve {
            // Check curve coefficients
            assert(4 * a * a * a + 27 * b * b != 0);

            let curve = Curve { a, b, gen };

            // gen should be on the curve
            assert(curve.contains(curve.gen));

            curve
        }

        // Conversion to affine coordinates
        pub fn into_affine(self) -> affine::Curve {
            let Curve { a, b, gen } = self;

            affine::Curve { a, b, gen: gen.into_affine() }
        }

        // Membership check
        pub fn contains(self, p: Point) -> bool {
            let Point { x, y, z } = p;
            if z == 0 {
                true
            } else {
                y * y == x * x * x + self.a * x * z * z * z * z + self.b * z * z * z * z * z * z
            }
        }

        // Addition
        pub fn add(self, p1: Point, p2: Point) -> Point {
            if p1.is_zero() {
                p2
            } else if p2.is_zero() {
                p1
            } else {
                let Point { x: x1, y: y1, z: z1 } = p1;
                let Point { x: x2, y: y2, z: z2 } = p2;
                let you1 = x1 * z2 * z2;
                let you2 = x2 * z1 * z1;
                let s1 = y1 * z2 * z2 * z2;
                let s2 = y2 * z1 * z1 * z1;

                if you1 == you2 {
                    if s1 != s2 {
                        Point::zero()
                    } else {
                        self.double(p1)
                    }
                } else {
                    let h = you2 - you1;
                    let r = s2 - s1;
                    let x3 = r * r - h * h * h - 2 * you1 * h * h;
                    let y3 = r * (you1 * h * h - x3) - s1 * h * h * h;
                    let z3 = h * z1 * z2;

                    Point::new(x3, y3, z3)
                }
            }
        }

        // Point doubling
        pub fn double(self, p: Point) -> Point {
            let Point { x, y, z } = p;

            if p.is_zero() {
                p
            } else if y == 0 {
                Point::zero()
            } else {
                let s = 4 * x * y * y;
                let m = 3 * x * x + self.a * z * z * z * z;
                let x0 = m * m - 2 * s;
                let y0 = m * (s - x0) - 8 * y * y * y * y;
                let z0 = 2 * y * z;

                Point::new(x0, y0, z0)
            }
        }

        // Scalar multiplication with scalar represented by a bit array (little-endian convention).
        // If k is the natural number represented by `bits`, then this computes p + ... + p k times.
        pub fn bit_mul<let N: u32>(self, bits: [u1; N], p: Point) -> Point {
            let mut out = Point::zero();

            for i in 0..N {
                out = self.add(
                    self.add(out, out),
                    if (bits[N - i - 1] == 0) {
                        Point::zero()
                    } else {
                        p
                    },
                );
            }

            out
        }

        // Scalar multiplication (p + ... + p n times)
        pub fn mul(self, n: Field, p: Point) -> Point {
            // TODO: temporary workaround until issue 1354 is solved
            let mut n_as_bits: [u1; 254] = [0; 254];
            let tmp: [u1; 254] = n.to_le_bits();
            for i in 0..254 {
                n_as_bits[i] = tmp[i];
            }

            self.bit_mul(n_as_bits, p)
        }

        // Multi-scalar multiplication (n[0]*p[0] + ... + n[N]*p[N], where * denotes scalar multiplication)
        pub fn msm<let N: u32>(self, n: [Field; N], p: [Point; N]) -> Point {
            let mut out = Point::zero();

            for i in 0..N {
                out = self.add(out, self.mul(n[i], p[i]));
            }

            out
        }

        // Point subtraction
        pub fn subtract(self, p1: Point, p2: Point) -> Point {
            self.add(p1, p2.negate())
        }

        // Simplified SWU map-to-curve method
        pub fn swu_map(self, z: Field, u: Field) -> Point {
            self.into_affine().swu_map(z, u).into_group()
        }
    }
}
