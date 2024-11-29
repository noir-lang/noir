pub mod affine {
    // Affine coordinate representation of Twisted Edwards curves
    // Points are represented by two-dimensional Cartesian coordinates.
    // Group operations are implemented in terms of those in CurveGroup (in this case, extended Twisted Edwards) coordinates
    // for reasons of efficiency.
    // See <https://eprint.iacr.org/2008/522.pdf> for details.
    use crate::cmp::Eq;
    use crate::ec::montcurve::affine::Curve as MCurve;
    use crate::ec::montcurve::affine::Point as MPoint;
    use crate::ec::swcurve::affine::Curve as SWCurve;
    use crate::ec::swcurve::affine::Point as SWPoint;
    use crate::ec::tecurve::curvegroup;

    // Curve specification
    pub struct Curve { // Twisted Edwards curve
        // Coefficients in defining equation ax^2 + y^2 = 1 + dx^2y^2
        pub a: Field,
        pub d: Field,
        // Generator as point in Cartesian coordinates
        pub gen: Point,
    }
    // Point in Cartesian coordinates
    pub struct Point {
        pub x: Field,
        pub y: Field,
    }

    impl Point {
        // Point constructor
        // #[deprecated("It's recommmended to use the external noir-edwards library (https://github.com/noir-lang/noir-edwards)")]
        pub fn new(x: Field, y: Field) -> Self {
            Self { x, y }
        }

        // Check if zero
        pub fn is_zero(self) -> bool {
            self.eq(Point::zero())
        }

        // Conversion to CurveGroup coordinates
        pub fn into_group(self) -> curvegroup::Point {
            let Self { x, y } = self;

            curvegroup::Point::new(x, y, x * y, 1)
        }

        // Additive identity
        pub fn zero() -> Self {
            Point::new(0, 1)
        }

        // Negation
        pub fn negate(self) -> Self {
            let Self { x, y } = self;
            Point::new(0 - x, y)
        }

        // Map into prime-order subgroup of equivalent Montgomery curve
        pub fn into_montcurve(self) -> MPoint {
            if self.is_zero() {
                MPoint::zero()
            } else {
                let Self { x, y } = self;
                let x0 = (1 + y) / (1 - y);
                let y0 = (1 + y) / (x * (1 - y));

                MPoint::new(x0, y0)
            }
        }
    }

    impl Eq for Point {
        fn eq(self, p: Self) -> bool {
            let Self { x: x1, y: y1 } = self;
            let Self { x: x2, y: y2 } = p;

            (x1 == x2) & (y1 == y2)
        }
    }

    impl Curve {
        // Curve constructor
        pub fn new(a: Field, d: Field, gen: Point) -> Curve {
            // Check curve coefficients
            assert(a * d * (a - d) != 0);

            let curve = Curve { a, d, gen };

            // gen should be on the curve
            assert(curve.contains(curve.gen));

            curve
        }

        // Conversion to CurveGroup coordinates
        pub fn into_group(self) -> curvegroup::Curve {
            let Curve { a, d, gen } = self;

            curvegroup::Curve { a, d, gen: gen.into_group() }
        }

        // Membership check
        pub fn contains(self, p: Point) -> bool {
            let Point { x, y } = p;
            self.a * x * x + y * y == 1 + self.d * x * x * y * y
        }

        // Point addition, implemented in terms of mixed addition for reasons of efficiency
        pub fn add(self, p1: Point, p2: Point) -> Point {
            self.mixed_add(p1, p2.into_group()).into_affine()
        }

        // Mixed point addition, i.e. first argument in affine, second in CurveGroup coordinates.
        pub fn mixed_add(self, p1: Point, p2: curvegroup::Point) -> curvegroup::Point {
            let Point { x: x1, y: y1 } = p1;
            let curvegroup::Point { x: x2, y: y2, t: t2, z: z2 } = p2;

            let a = x1 * x2;
            let b = y1 * y2;
            let c = self.d * x1 * y1 * t2;
            let e = (x1 + y1) * (x2 + y2) - a - b;
            let f = z2 - c;
            let g = z2 + c;
            let h = b - self.a * a;

            let x = e * f;
            let y = g * h;
            let t = e * h;
            let z = f * g;

            curvegroup::Point::new(x, y, t, z)
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

        // Conversion to equivalent Montgomery curve
        pub fn into_montcurve(self) -> MCurve {
            let j = 2 * (self.a + self.d) / (self.a - self.d);
            let k = 4 / (self.a - self.d);
            let gen_montcurve = self.gen.into_montcurve();

            MCurve::new(j, k, gen_montcurve)
        }

        // Conversion to equivalent Short Weierstrass curve
        pub fn into_swcurve(self) -> SWCurve {
            self.into_montcurve().into_swcurve()
        }

        // Point mapping into equivalent Short Weierstrass curve
        pub fn map_into_swcurve(self, p: Point) -> SWPoint {
            self.into_montcurve().map_into_swcurve(p.into_montcurve())
        }

        // Point mapping from equivalent Short Weierstrass curve
        pub fn map_from_swcurve(self, p: SWPoint) -> Point {
            self.into_montcurve().map_from_swcurve(p).into_tecurve()
        }

        // Elligator 2 map-to-curve method (via rational map)
        pub fn elligator2_map(self, u: Field) -> Point {
            self.into_montcurve().elligator2_map(u).into_tecurve()
        }

        // Simplified SWU map-to-curve method (via rational map)
        pub fn swu_map(self, z: Field, u: Field) -> Point {
            self.into_montcurve().swu_map(z, u).into_tecurve()
        }
    }
}
pub mod curvegroup {
    // CurveGroup coordinate representation of Twisted Edwards curves
    // Points are represented by four-dimensional projective coordinates, viz. extended Twisted Edwards coordinates.
    // See section 3 of <https://eprint.iacr.org/2008/522.pdf> for details.
    use crate::cmp::Eq;
    use crate::ec::montcurve::curvegroup::Curve as MCurve;
    use crate::ec::montcurve::curvegroup::Point as MPoint;
    use crate::ec::swcurve::curvegroup::Curve as SWCurve;
    use crate::ec::swcurve::curvegroup::Point as SWPoint;
    use crate::ec::tecurve::affine;

    // Curve specification
    pub struct Curve { // Twisted Edwards curve
        // Coefficients in defining equation a(x^2 + y^2)z^2 = z^4 + dx^2y^2
        pub a: Field,
        pub d: Field,
        // Generator as point in projective coordinates
        pub gen: Point,
    }
    // Point in extended twisted Edwards coordinates
    pub struct Point {
        pub x: Field,
        pub y: Field,
        pub t: Field,
        pub z: Field,
    }

    impl Point {
        // Point constructor
        pub fn new(x: Field, y: Field, t: Field, z: Field) -> Self {
            Self { x, y, t, z }
        }

        // Check if zero
        pub fn is_zero(self) -> bool {
            let Self { x, y, t, z } = self;
            (x == 0) & (y == z) & (y != 0) & (t == 0)
        }

        // Conversion to affine coordinates
        pub fn into_affine(self) -> affine::Point {
            let Self { x, y, t: _t, z } = self;

            affine::Point::new(x / z, y / z)
        }

        // Additive identity
        pub fn zero() -> Self {
            Point::new(0, 1, 0, 1)
        }

        // Negation
        pub fn negate(self) -> Self {
            let Self { x, y, t, z } = self;

            Point::new(0 - x, y, 0 - t, z)
        }

        // Map into prime-order subgroup of equivalent Montgomery curve
        pub fn into_montcurve(self) -> MPoint {
            self.into_affine().into_montcurve().into_group()
        }
    }

    impl Eq for Point {
        fn eq(self, p: Self) -> bool {
            let Self { x: x1, y: y1, t: _t1, z: z1 } = self;
            let Self { x: x2, y: y2, t: _t2, z: z2 } = p;

            (x1 * z2 == x2 * z1) & (y1 * z2 == y2 * z1)
        }
    }

    impl Curve {
        // Curve constructor
        pub fn new(a: Field, d: Field, gen: Point) -> Curve {
            // Check curve coefficients
            assert(a * d * (a - d) != 0);

            let curve = Curve { a, d, gen };

            // gen should be on the curve
            assert(curve.contains(curve.gen));

            curve
        }

        // Conversion to affine coordinates
        pub fn into_affine(self) -> affine::Curve {
            let Curve { a, d, gen } = self;

            affine::Curve { a, d, gen: gen.into_affine() }
        }

        // Membership check
        pub fn contains(self, p: Point) -> bool {
            let Point { x, y, t, z } = p;

            (z != 0)
                & (z * t == x * y)
                & (z * z * (self.a * x * x + y * y) == z * z * z * z + self.d * x * x * y * y)
        }

        // Point addition
        pub fn add(self, p1: Point, p2: Point) -> Point {
            let Point { x: x1, y: y1, t: t1, z: z1 } = p1;
            let Point { x: x2, y: y2, t: t2, z: z2 } = p2;

            let a = x1 * x2;
            let b = y1 * y2;
            let c = self.d * t1 * t2;
            let d = z1 * z2;
            let e = (x1 + y1) * (x2 + y2) - a - b;
            let f = d - c;
            let g = d + c;
            let h = b - self.a * a;

            let x = e * f;
            let y = g * h;
            let t = e * h;
            let z = f * g;

            Point::new(x, y, t, z)
        }

        // Point doubling, cf. section 3.3
        pub fn double(self, p: Point) -> Point {
            let Point { x, y, t: _t, z } = p;

            let a = x * x;
            let b = y * y;
            let c = 2 * z * z;
            let d = self.a * a;
            let e = (x + y) * (x + y) - a - b;
            let g = d + b;
            let f = g - c;
            let h = d - b;

            let x0 = e * f;
            let y0 = g * h;
            let t0 = e * h;
            let z0 = f * g;

            Point::new(x0, y0, t0, z0)
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

        // Conversion to equivalent Montgomery curve
        pub fn into_montcurve(self) -> MCurve {
            self.into_affine().into_montcurve().into_group()
        }

        // Conversion to equivalent Short Weierstrass curve
        pub fn into_swcurve(self) -> SWCurve {
            self.into_montcurve().into_swcurve()
        }

        // Point mapping into equivalent short Weierstrass curve
        pub fn map_into_swcurve(self, p: Point) -> SWPoint {
            self.into_montcurve().map_into_swcurve(p.into_montcurve())
        }

        // Point mapping from equivalent short Weierstrass curve
        pub fn map_from_swcurve(self, p: SWPoint) -> Point {
            self.into_montcurve().map_from_swcurve(p).into_tecurve()
        }

        // Elligator 2 map-to-curve method (via rational maps)
        pub fn elligator2_map(self, u: Field) -> Point {
            self.into_montcurve().elligator2_map(u).into_tecurve()
        }

        // Simplified SWU map-to-curve method (via rational map)
        pub fn swu_map(self, z: Field, u: Field) -> Point {
            self.into_montcurve().swu_map(z, u).into_tecurve()
        }
    }
}
