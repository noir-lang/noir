// https://github.com/noir-lang/noir/issues/7749

fn main() {
    let pt_y = 17631683881184975370165255887551781615748388533673675138860;
    let pt = std::embedded_curve_ops::EmbeddedCurvePoint { x: 1, y: pt_y, is_infinite: false };
    let pt_2x = pt.double();
    let pt_4x = pt_2x.double();
    assert(pt_2x != pt_4x);
}
