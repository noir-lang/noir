use std::embedded_curve_ops::{EmbeddedCurvePoint, EmbeddedCurveScalar, fixed_base_scalar_mul};

fn main() -> pub Field {
    let pre_address = 0x23d95e303879a5d0bbef78ecbc335e559da37431f6dcd11da54ed375c2846813;
    let (a, b) = std::field::bn254::decompose(pre_address);
    let curve = EmbeddedCurveScalar { lo: a, hi: b };
    let key = fixed_base_scalar_mul(curve);
    let point = EmbeddedCurvePoint {
        x: 0x111223493147f6785514b1c195bb37a2589f22a6596d30bb2bb145fdc9ca8f1e,
        y: 0x273bbffd678edce8fe30e0deafc4f66d58357c06fd4a820285294b9746c3be95,
        is_infinite: false,
    };
    let address_point = key.add(point);
    address_point.x
}
