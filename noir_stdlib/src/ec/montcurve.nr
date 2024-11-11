pub mod affine {
    // Affine representation of Montgomery curves
    // Points are represented by two-dimensional Cartesian coordinates.
    // All group operations are induced by those of the corresponding Twisted Edwards curve.
    // See e.g. <https://eprint.iacr.org/2017/212.pdf> for details on the correspondences.
    use crate::cmp::Eq;
    use crate::ec::is_square;
    use crate::ec::montcurve::curvegroup;
    use crate::ec::safe_inverse;
    use crate::ec::sqrt;
    use crate::ec::swcurve::affine::Curve as SWCurve;
    use crate::ec::swcurve::affine::Point as SWPoint;
    use crate::ec::tecurve::affine::Curve as TECurve;
    use crate::ec::tecurve::affine::Point as TEPoint;
    use crate::ec::ZETA;

    // Curve specification
    pub struct Curve { // Montgomery Curve configuration (ky^2 = x^3 + j*x^2 + x)
        pub j: Field,
        pub k: Field,
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
            self.infty
        }

        // Conversion to CurveGroup coordinates
        pub fn into_group(self) -> curvegroup::Point {
            if self.is_zero() {
                curvegroup::Point::zero()
            } else {
                let (x, y) = (self.x, self.y);
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

        // Map into equivalent Twisted Edwards curve
        pub fn into_tecurve(self) -> TEPoint {
            let Self { x, y, infty } = self;

            if infty | (y * (x + 1) == 0) {
                TEPoint::zero()
            } else {
                TEPoint::new(x / y, (x - 1) / (x + 1))
            }
        }
    }

    impl Eq for Point {
        fn eq(self, p: Self) -> bool {
            (self.infty & p.infty) | (!self.infty & !p.infty & (self.x == p.x) & (self.y == p.y))
        }
    }

    impl Curve {
        // Curve constructor
        pub fn new(j: Field, k: Field, gen: Point) -> Self {
            // Check curve coefficients
            assert(k != 0);
            assert(j * j != 4);

            let curve = Self { j, k, gen };

            // gen should be on the curve
            assert(curve.contains(curve.gen));

            curve
        }

        // Conversion to CurveGroup coordinates
        pub fn into_group(self) -> curvegroup::Curve {
            curvegroup::Curve::new(self.j, self.k, self.gen.into_group())
        }

        // Membership check
        pub fn contains(self, p: Point) -> bool {
            let Self { j, k, gen: _gen } = self;
            let Point { x, y, infty } = p;

            infty | (k * y * y == x * (x * x + j * x + 1))
        }

        // Point addition
        pub fn add(self, p1: Point, p2: Point) -> Point {
            self.into_tecurve().add(p1.into_tecurve(), p2.into_tecurve()).into_montcurve()
        }

        // Scalar multiplication with scalar represented by a bit array (little-endian convention).
        // If k is the natural number represented by `bits`, then this computes p + ... + p k times.
        pub fn bit_mul<let N: u32>(self, bits: [u1; N], p: Point) -> Point {
            self.into_tecurve().bit_mul(bits, p.into_tecurve()).into_montcurve()
        }

        // Scalar multiplication (p + ... + p n times)
        pub fn mul(self, n: Field, p: Point) -> Point {
            self.into_tecurve().mul(n, p.into_tecurve()).into_montcurve()
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

        // Conversion to equivalent Twisted Edwards curve
        pub fn into_tecurve(self) -> TECurve {
            let Self { j, k, gen } = self;
            TECurve::new((j + 2) / k, (j - 2) / k, gen.into_tecurve())
        }

        // Conversion to equivalent Short Weierstrass curve
        pub fn into_swcurve(self) -> SWCurve {
            let j = self.j;
            let k = self.k;
            let a0 = (3 - j * j) / (3 * k * k);
            let b0 = (2 * j * j * j - 9 * j) / (27 * k * k * k);

            SWCurve::new(a0, b0, self.map_into_swcurve(self.gen))
        }

        // Point mapping into equivalent Short Weierstrass curve
        pub fn map_into_swcurve(self, p: Point) -> SWPoint {
            if p.is_zero() {
                SWPoint::zero()
            } else {
                SWPoint::new((3 * p.x + self.j) / (3 * self.k), p.y / self.k)
            }
        }

        // Point mapping from equivalent Short Weierstrass curve
        pub fn map_from_swcurve(self, p: SWPoint) -> Point {
            let SWPoint { x, y, infty } = p;
            let j = self.j;
            let k = self.k;

            Point { x: (3 * k * x - j) / 3, y: y * k, infty }
        }

        // Elligator 2 map-to-curve method; see <https://datatracker.ietf.org/doc/id/draft-irtf-cfrg-hash-to-curve-06.html#name-elligator-2-method>.
        pub fn elligator2_map(self, u: Field) -> Point {
            let j = self.j;
            let k = self.k;
            let z = ZETA; // Non-square Field element required for map
            // Check whether curve is admissible
            assert(j != 0);
            let l = (j * j - 4) / (k * k);
            assert(l != 0);
            assert(is_square(l) == false);

            let x1 = safe_inverse(1 + z * u * u) * (0 - (j / k));

            let gx1 = x1 * x1 * x1 + (j / k) * x1 * x1 + x1 / (k * k);
            let x2 = 0 - x1 - (j / k);
            let gx2 = x2 * x2 * x2 + (j / k) * x2 * x2 + x2 / (k * k);

            let x = if is_square(gx1) { x1 } else { x2 };

            let y = if is_square(gx1) {
                let y0 = sqrt(gx1);
                if y0.sgn0() == 1 {
                    y0
                } else {
                    0 - y0
                }
            } else {
                let y0 = sqrt(gx2);
                if y0.sgn0() == 0 {
                    y0
                } else {
                    0 - y0
                }
            };

            Point::new(x * k, y * k)
        }

        // SWU map-to-curve method (via rational map)
        pub fn swu_map(self, z: Field, u: Field) -> Point {
            self.map_from_swcurve(self.into_swcurve().swu_map(z, u))
        }
    }
}
pub mod curvegroup {
    // Affine representation of Montgomery curves
    // Points are represented by three-dimensional projective (homogeneous) coordinates.
    // All group operations are induced by those of the corresponding Twisted Edwards curve.
    // See e.g. <https://eprint.iacr.org/2017/212.pdf> for details on the correspondences.
    use crate::cmp::Eq;
    use crate::ec::montcurve::affine;
    use crate::ec::swcurve::curvegroup::Curve as SWCurve;
    use crate::ec::swcurve::curvegroup::Point as SWPoint;
    use crate::ec::tecurve::curvegroup::Curve as TECurve;
    use crate::ec::tecurve::curvegroup::Point as TEPoint;

    pub struct Curve { // Montgomery Curve configuration (ky^2 z = x*(x^2 + j*x*z + z*z))
        pub j: Field,
        pub k: Field,
        // Generator as point in projective coordinates
        pub gen: Point,
    }
    // Point in projective coordinates
    pub struct Point {
        pub x: Field,
        pub y: Field,
        pub z: Field,
    }

    impl Point {
        // Point constructor
        pub fn new(x: Field, y: Field, z: Field) -> Self {
            Self { x, y, z }
        }

        // Check if zero
        pub fn is_zero(self) -> bool {
            self.z == 0
        }

        // Conversion to affine coordinates
        pub fn into_affine(self) -> affine::Point {
            if self.is_zero() {
                affine::Point::zero()
            } else {
                let (x, y, z) = (self.x, self.y, self.z);
                affine::Point::new(x / z, y / z)
            }
        }

        // Additive identity
        pub fn zero() -> Self {
            Self { x: 0, y: 1, z: 0 }
        }

        // Negation
        pub fn negate(self) -> Self {
            let Self { x, y, z } = self;

            Point::new(x, 0 - y, z)
        }

        // Map into equivalent Twisted Edwards curve
        pub fn into_tecurve(self) -> TEPoint {
            self.into_affine().into_tecurve().into_group()
        }
    }

    impl Eq for Point {
        fn eq(self, p: Self) -> bool {
            (self.z == p.z)
                | (((self.x * self.z) == (p.x * p.z)) & ((self.y * self.z) == (p.y * p.z)))
        }
    }

    impl Curve {
        // Curve constructor
        pub fn new(j: Field, k: Field, gen: Point) -> Self {
            // Check curve coefficients
            assert(k != 0);
            assert(j * j != 4);

            let curve = Self { j, k, gen };

            // gen should be on the curve
            assert(curve.contains(curve.gen));

            curve
        }

        // Conversion to affine coordinates
        pub fn into_affine(self) -> affine::Curve {
            affine::Curve::new(self.j, self.k, self.gen.into_affine())
        }

        // Membership check
        pub fn contains(self, p: Point) -> bool {
            let Self { j, k, gen: _gen } = self;
            let Point { x, y, z } = p;

            k * y * y * z == x * (x * x + j * x * z + z * z)
        }

        // Point addition
        pub fn add(self, p1: Point, p2: Point) -> Point {
            self.into_affine().add(p1.into_affine(), p2.into_affine()).into_group()
        }

        // Scalar multiplication with scalar represented by a bit array (little-endian convention).
        // If k is the natural number represented by `bits`, then this computes p + ... + p k times.
        pub fn bit_mul<let N: u32>(self, bits: [u1; N], p: Point) -> Point {
            self.into_tecurve().bit_mul(bits, p.into_tecurve()).into_montcurve()
        }

        // Scalar multiplication (p + ... + p n times)
        pub fn mul(self, n: Field, p: Point) -> Point {
            self.into_tecurve().mul(n, p.into_tecurve()).into_montcurve()
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

        // Conversion to equivalent Twisted Edwards curve
        pub fn into_tecurve(self) -> TECurve {
            let Self { j, k, gen } = self;
            TECurve::new((j + 2) / k, (j - 2) / k, gen.into_tecurve())
        }

        // Conversion to equivalent Short Weierstrass curve
        pub fn into_swcurve(self) -> SWCurve {
            let j = self.j;
            let k = self.k;
            let a0 = (3 - j * j) / (3 * k * k);
            let b0 = (2 * j * j * j - 9 * j) / (27 * k * k * k);

            SWCurve::new(a0, b0, self.map_into_swcurve(self.gen))
        }

        // Point mapping into equivalent Short Weierstrass curve
        pub fn map_into_swcurve(self, p: Point) -> SWPoint {
            self.into_affine().map_into_swcurve(p.into_affine()).into_group()
        }

        // Point mapping from equivalent Short Weierstrass curve
        pub fn map_from_swcurve(self, p: SWPoint) -> Point {
            self.into_affine().map_from_swcurve(p.into_affine()).into_group()
        }

        // Elligator 2 map-to-curve method
        pub fn elligator2_map(self, u: Field) -> Point {
            self.into_affine().elligator2_map(u).into_group()
        }

        // SWU map-to-curve method (via rational map)
        pub fn swu_map(self, z: Field, u: Field) -> Point {
            self.into_affine().swu_map(z, u).into_group()
        }
    }
}
