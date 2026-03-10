pub struct T256Point {
    pub x: Field,
    pub y: Field,
}

pub struct T256Scalar {
    pub lo: Field,
    pub hi: Field,
}

pub fn t256_ec_add(p1: T256Point, p2: T256Point) -> T256Point {
    t256_ec_add_array_return(p1, p2, true)[0]
}

pub fn t256_msm<let N: u32>(points: [T256Point; N], scalars: [T256Scalar; N]) -> T256Point {
    t256_msm_array_return(points, scalars, true)[0]
}

#[foreign(embedded_curve_add)]
fn t256_ec_add_array_return(
    _p1: T256Point,
    _p2: T256Point,
    _predicate: bool,
) -> [T256Point; 1] {}

#[foreign(multi_scalar_mul)]
fn t256_msm_array_return<let N: u32>(
    _points: [T256Point; N],
    _scalars: [T256Scalar; N],
    _predicate: bool,
) -> [T256Point; 1] {}