mod scalar_field;
mod test;
mod bjj;

use dep::std;
use crate::scalar_field::ScalarField;

struct Curve<Params> {
    x: Field,
    y: Field,
}

// ####################################################################################################################
// ####################################################################################################################
// ### T R A I T S
// ####################################################################################################################
// ####################################################################################################################
/**
 * @brief parametrises a Twisted Edwards curve
 **/
trait TECurveParameterTrait {
    fn a() -> Field; // twisted edward curve parameter a
    fn d() -> Field; // twisted edward curve parameter d
    fn gen() -> (Field, Field); // generator point x/y coordinates
}

/**
 * @brief defines methods that a valid Curve implementation must satisfy
 **/
trait CurveTrait<Params> where CurveTrait<Params>: std::ops::Add + std::ops::Sub + std::ops::Eq + std::ops::Neg  {
    fn default() -> Self { std::default::Default::default  () }
    fn new(x: Field, y: Field) -> Self;
    fn zero() -> Self;
    fn one() -> Self;

    fn add(self, x: Self) -> Self { self + x }
    fn sub(self, x: Self) -> Self { self - x }
    fn neg(self) -> Self { std::ops::Neg::neg(self) }
    fn dbl(self) -> Self;
    fn mul<NScalarSlices>(self, x: ScalarField<NScalarSlices>) -> Self;
    fn msm<N, NScalarSlices> (points: [Self; N], scalars: [ScalarField<NScalarSlices>; N]) -> Self;

    fn eq(self, x: Self) -> bool { self == x }
    fn is_zero(self) -> bool { self == Self::zero() }

    fn is_on_curve(self) -> bool;
    fn assert_is_on_curve(self);
    fn assert_equal(self, other: Self);
}

// ####################################################################################################################
// ####################################################################################################################
// ### C O N S T R A I N E D    F U N C T I O N S
// ####################################################################################################################
// ####################################################################################################################
impl<Params> std::default::Default for Curve<Params> where Params: TECurveParameterTrait {
    /**
     * @brief return point at infinity
     *
     * Cost: 0 gates
     **/
    fn default() -> Self {
        Curve::zero()
    }
}

impl<Params> std::ops::Add for Curve<Params> where Params: TECurveParameterTrait {
    /**
     * @brief compute `self + other`
     *
     * Cost: 7 gates
     **/
    fn add(self, other:Self) -> Self {
        Curve::add_internal(self, other, Params::a(), Params::d())
    }
}

impl<Params> std::ops::Neg for Curve<Params> where Params: TECurveParameterTrait {
    /**
     * @brief negate a point
     *
     * Cost: usually 0, will cost 1 gate if the `x` coordinate needs to be converted into a witness
     **/
    fn neg(self) -> Self {
        Curve{ x: -self.x, y: self.y }
    }
}

impl<Params> std::ops::Sub for Curve<Params> where Params: TECurveParameterTrait {
    /**
     * @brief compute `self - other`
     *
     * Cost: 7 gates
     **/
    fn sub(self, other:Self) -> Self {
        Curve::add_internal(self, other.neg(), Params::a(), Params::d())
    }
}

impl<Params> std::cmp::Eq for Curve<Params> where Params: TECurveParameterTrait {
    /**
     * @brief compute `self == other`
     *
     * Cost: 6 gates
     **/
    fn eq(self, other:Self) -> bool {
        (self.x == other.x) & (self.y == other.y)
    }
}

impl<Params> std::convert::From<(Field, Field)> for Curve<Params> where Params: TECurveParameterTrait {
    /**
     * @brief construct from tuple of field elements
     * @details use this method instead of `new` if you know x/y is on the curve
     *
     * Cost: 0 gates
     **/
    fn from((x, y): (Field, Field)) -> Self {
        Curve{ x, y }
    }
}

impl<Params> CurveTrait<Params> for Curve<Params> where Params: TECurveParameterTrait {

    /**
     * @brief Construct a new point
     * @details If you know the x/y coords form a valid point DO NOT USE THIS METHOD
     *          This method calls `assert_is_on_curve` which costs 3 gates.
     *          Instead, directly construct via Curve{x, y}
     *          Or use from((x, y))
     *
     * Cost: 3 gates
     **/
    fn new(x: Field, y: Field) -> Self {
        let result = Curve{ x, y };
        result.assert_is_on_curve();
        result
    }

    /**
     * @brief return the Identity element (point at infinity)
     *
     * Cost: 0 gates
     **/
    fn zero() -> Self { Curve{ x: 0, y: 1 } }

    /**
     * @brief return the Generator of the group
     *
     * Cost: 0 gates (assuming Params trait returns values known at compile time!)
     **/
    fn one() -> Self { 
        let (x, y) = Params::gen();
        Curve{ x, y }
    }

    /**
     * @brief validate a point is on the curve
     * @details cheaper than `is_on_curve` (assert is cheaper than returning a bool)
     *
     * Cost: 3 gates
     **/
    fn assert_is_on_curve(self) {
        let t0 = self.x * self.x;
        let t1 = self.y * self.y;
        std::as_witness(t0);
        std::as_witness(t1);
        let t2 = Params::a() * t0 + t1;
        let t3 = 1 + Params::d() * t0 * t1;
        assert(t2 == t3);
    }

    /**
     * @brief Constrain two points to equal each other
     * @details Cheaper than `assert(self == other)` because no need to return a bool
     *
     * Cost: 0-2 gates (can do these asserts with just copy constraints)
     **/
    fn assert_equal(self, other: Self) {
        assert(self.x == other.x);
        assert(self.y == other.y);
    }

    /**
     * @brief return a bool that describes whether the point is on the curve
     * @details if you don't need to handle the failure case, it is cheaper to call `assert_is_on_curve`
     *
     * Cost: 5 gates
     **/
    fn is_on_curve(self) -> bool {
        let t0 = self.x * self.x;
        let t1 = self.y * self.y;
        std::as_witness(t0);
        std::as_witness(t1);
        let t2 = Params::a() * t0 + t1;
        let t3 = 1 + Params::d() * t0 * t1;
        (t2 == t3)
    } 

    /**
     * @brief compute `self + self`
     *
     * Cost: 5 gates
     **/
    fn dbl(self) -> Self {
        Curve::dbl_internal(self, Params::a(), Params::d())
    }

    /**
     * @brief compute `self * scalar`
     * @details uses the Straus method via lookup tables.
     *          Assumes backend has an efficient implementation of a memory table abstraction
     *          i.e. `let x = table[y]` is efficient even if `y` is not known at compile time
     * 
     * Key cost components are as follows:
     *      1: computing the Straus point lookup table (169 gates)
     *      2: 252 point doublings (1260 gates)
     *      3: 63 point additions (441 gates)
     *      4: 126 table reads with runtime index (252 gates)
     *
     * Cost: 2122 gates + cost of creating ScalarField (110 gates)
     *
     * TODO: use windowed non-adjacent form to remove 7 point additions when creating lookup table
     **/
    fn mul<NScalarSlices>(self: Self, scalar: ScalarField<NScalarSlices>) -> Self {
        // define a, d params locally to make code more readable (shouldn't affect performance) 
        let a = Params::a();
        let d = Params::d();

        // Construct tables of precomputed point coordinates.
        let (table_x, table_y): ([Field; 16], [Field; 16]) = self.compute_straus_point_table(a, d);

        // Initialize the accumulator with the point that maps to the first (most significant) scalar slice
        let idx = scalar.base4_slices[0];
        let mut accumulator: Self = Curve { x: table_x[idx], y: table_y[idx] };

        // Execute a double-and-add subroutine
        // 1. Compute `accumulator = accumulator * 16`
        // 2. Extract 4-bits from the scalar multiplier and
        //    use them to retrieve the corresponding point from our point table
        // Note: this is similar to the "double and add" scalar multiplication method, except we use base16 instead of base2!
        for i in 1..NScalarSlices {
            accumulator = accumulator.dbl_internal(a, d);
            accumulator = accumulator.dbl_internal(a, d);
            accumulator = accumulator.dbl_internal(a, d);
            accumulator = accumulator.dbl_internal(a, d);
            let idx: u8 = scalar.base4_slices[i];
            let x = table_x[idx];
            let y = table_y[idx];
            accumulator = accumulator.add_internal(Curve{ x, y }, a, d);
        }

        // todo fix
        if (scalar.skew) {
            accumulator = accumulator - self;
        }
        accumulator
    }

    /**
     * @brief compute `points[0] * scalar[0] + ... + points[N-1] * scalar[N-1]`
     * @details Is cheaper than `mul` when processing >1 point due to reduced number of point doublings
     *          uses the Straus MSM method via lookup tables.
     *          Assumes backend has an efficient implementation of a memory table abstraction
     *          i.e. `let x = table[y]` is efficient even if `y` is not known at compile time
     * 
     * Key cost components are as follows
     * PER POINT costs:
     *      1: computing the Straus point lookup table (169N gates)
     *      2: 63 point additions (441N gates)
     *      3: 126 table reads with runtime index (252N gates)
     *
     * Additional costs:
     *      1. 252 point doublings 1260 gates
     *
     * Cost: 1260 + 862N + cost of creating ScalarField (110N gates)
     *
     * TODO: use windowed non-adjacent form to remove 7 point additions per point when creating lookup table
     **/
    fn msm<N, NScalarSlices> (points: [Self; N], scalars: [ScalarField<NScalarSlices>; N]) -> Self
    {
        let a = Params::a();
        let d = Params::d();

        // Generalized version of `mul` for multiple points.
        let mut point_tables: [([Field; 16], [Field; 16]); N] = [([0; 16], [0; 16]); N];
        for j in 0..N {
            point_tables[j] = points[j].compute_straus_point_table(a, d);
        }

        let idx = scalars[0].base4_slices[0];
        let mut accumulator: Self = Curve{ x: point_tables[0].0[idx], y: point_tables[0].1[idx] };
        for j in 1..N {
            let idx = scalars[j].base4_slices[0];
            let P = Curve{ x: point_tables[j].0[idx], y: point_tables[j].1[idx] };
            accumulator = accumulator.add_internal(P, a, d);
        }
        for i in 1..NScalarSlices {
            accumulator = accumulator.dbl_internal(a, d);
            accumulator = accumulator.dbl_internal(a, d);
            accumulator = accumulator.dbl_internal(a, d);
            accumulator = accumulator.dbl_internal(a, d);
            for j in 0..N {
                let idx: u8 = scalars[j].base4_slices[i];
                let x = point_tables[j].0[idx];
                let y = point_tables[j].1[idx];
                accumulator = accumulator.add_internal(Curve{ x, y }, a, d);
            }
        }

        for j in 0..N {
            if (scalars[j].skew == true)
            {
                accumulator = accumulator - points[j];
            }
        }
        accumulator
    }
}

// ####################################################################################################################
// ####################################################################################################################
// ### H E L P E R    F U N C T I O N S
// ####################################################################################################################
// ####################################################################################################################
impl<Params> Curve<Params> {
    /**
     * @brief add points together, return output + lambda term
     **/
    unconstrained pub fn __add_unconstrained(x1: Field, x2: Field, y1: Field, y2: Field, a: Field, d: Field) -> (Field, Field, Field) {
        let lambda = y1 * y2 * x1 * x2;
        let y = (x1 * x2 * a - y1 * y2) / (lambda * d - 1);
        let x = (x1 * y2 + y1 * x2) / (lambda * d + 1);
        (x, y, lambda)
    }

    /**
     * @brief add two points together
     * @details This method exists because of a Noir bug where `Params` cannot be accessed by an internal function
     *          called from internal function.
     *          e.g. compiler error if `mul` impl tries to call `add` :(
     **/
    fn add_internal(self, other: Self, a: Field, d: Field) -> Self {
        let x1 = self.x;
        let x2 = other.x;
        let y1 = self.y;
        let y2 = other.y;
        let (x, y, lambda) = Curve::__add_unconstrained(x1, x2, y1, y2, a, d);
        let x1x2 = x1 * x2;
        let x1y2 = x1 * y2;
        std::as_witness(x1x2);
        std::as_witness(x1y2);
        let x_lhs = x * lambda * d + x - x1y2; // equals y1x2
        let y_lhs = y * lambda * -d + y + x1x2 * a; // equals y1y2
        let y1x2 = y1 * x2;
        let y1y2 = y1 * y2;
        std::as_witness(y1x2);
        std::as_witness(y1y2);
        let y1y2x1x2 = y1y2 * x1x2;
        assert(x_lhs == y1x2);
        assert(y_lhs == y1y2);
        assert(y1y2x1x2 == lambda);
        Self { x, y }
    }

    /**
     * @brief add a point to itself
     * @details This method exists because of a Noir bug where `Params` cannot be accessed by an internal function
     *          called from internal function.
     *          e.g. compiler error if `mul` impl tries to call `dbl` :(
     **/
    fn dbl_internal(self, a: Field, d: Field) -> Self {
        let x1 = self.x;
        let y1 = self.y;
        let (x3, y3, _) = Curve::__add_unconstrained(x1, x1, y1, y1, a, d);
        let x1x1a = x1 * x1 * a;
        std::as_witness(x1x1a);
        // t1 = a*x_1^2 + y_1^2
        let t1 = y1 * y1 + x1x1a;
        std::as_witness(t1);
        // t3 = y_3 * (2 - a*x_1^2 + y_1^2) + 2*a*x_1^2
        let t3 = y3 + y3 - t1 * y3 + x1x1a * 2;
        // t3 == t1 implies y_3 * (2 - a*x_1^2 - y_1^2) + 2*a*x_1^2 == a*x_1^2 + y_1^2
        // i.e. y_3 = y_1^2 - a*x_1^2 / (2 - a*x_1^2 - y_1^2)
        assert(t3 == t1);
        let t4 = x1 * y1;
        std::as_witness(t4);
        // x3 * t1 - t4 == t4 implies x_3 * (y_1^2 + a * x_1^2) = 2 * x_1 * y_1
        // i.e. x_3 = 2 * x_1 * y_1 / (y_1^2 + a * x_1^2)
        let t2 = x3 * t1 - t4;
        assert(t2 == t4);
        Self { x: x3, y: y3 }
    }

    /**
     * @brief Compute a 4-bit lookup table of point multiples for the Straus windowed scalar multiplication algorithm.
     * @details Table contains [0, P, 2P, ..., 15P], which is used in the scalar mul algorithm to minimize the total number of required point additions
     *
     * @note It is cheaper to use ([Field; 16], [Field; 16]) than it is ([Curve; 16]).
     *       This is because the compiler will represent [Curve; 16] in 1 ROM array (vs 2 for [Field; 16], [Field; 16]).
     *       This means that any index into the ROM array for [Curve; 16] requires an additional arithmetic gate to process.
     *
     *          For example consider `let P: Curve = table[idx]`
     *          `table` will be a ROM array with 32 elements in it.
     *          The x-coordinates will be located at `2 * idx`
     *          The y-coordinates will be located at `2 * idx + 1`
     *          If `idx` is not known at compile time (for Straus it isnt), 2 arithmetic gates are required to evaluate `2 * idx`, `2 * idx + 1`
     *          before they can be used as arguments in a memory lookup protocol
     *
     *          Now consider `let P_x = table_x[idx]; let P_y = table_y[idx]`
     *          In this example, `idx` can be directly used as the argument into a memory lookup protocol for both tables.
     *
     *          For the Barretenberg backend, the cost of a Read-Only memory lookup is 2 gates,
     *          so splitting the x/y coordinates into separate tables means that the cost to lookup a point is 4 gates
     *          2 extra arithmetic gates would increase the cost by 50%, which we avoid by returning `([Field; 16], [Field; 16])` instead of `([Curve; 16])`
     *
     * Key cost components are as follows:
     *      1: Defining two size-16 lookup tables (2 gates per element, 32 elements = 64 gates)
     *      2: 15 point additions (7 * 5 = 105)
     *
     * Total Cost: 169 gates
     *
     * TODO: use windowed non-adjacent form to remove 8 point additions
     **/
    fn compute_straus_point_table(self, a: Field, d: Field) -> ([Field; 16], [Field; 16]) {
        let mut table_x: [Field; 16] = [0; 16];
        let mut table_y: [Field; 16] = [0; 16];
        table_x[8] = self.x;
        table_y[8] = self.y;
        let D = self.dbl_internal(a, d);
        for i in 1..8 {
            let Q = Curve { x: table_x[7 + i], y: table_y[7 + i] };
            let V = D.add_internal(Q, a, d);
            table_x[8 + i] = V.x;
            table_y[8 + i] = V.y;
        }
        for i in 0..8 {
            table_x[i] = -table_x[15 - i];
            table_y[i] = table_y[15 - i];
        }
        (table_x, table_y)
    }
}
