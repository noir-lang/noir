use std::embedded_curve_ops::EmbeddedCurvePoint;

// Note: If main has any unsized types, then the verifier will never be able
// to figure out the circuit instance
fn main() {
    let message = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let pub_key = EmbeddedCurvePoint {
        x: 0x04b260954662e97f00cab9adb773a259097f7a274b83b113532bce27fa3fb96a,
        y: 0x2fd51571db6c08666b0edfbfbc57d432068bccd0110a39b166ab243da0037197,
        is_infinite: false,
    };
    let signature = [
        1, 13, 119, 112, 212, 39, 233, 41, 84, 235, 255, 93, 245, 172, 186, 83, 157, 253, 76, 77,
        33, 128, 178, 15, 214, 67, 105, 107, 177, 234, 77, 48, 27, 237, 155, 84, 39, 84, 247, 27,
        22, 8, 176, 230, 24, 115, 145, 220, 254, 122, 135, 179, 171, 4, 214, 202, 64, 199, 19, 84,
        239, 138, 124, 12,
    ];

    let valid_signature = std::schnorr::verify_signature(pub_key, signature, message);
    assert(valid_signature);
}
