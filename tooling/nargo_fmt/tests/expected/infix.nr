//@error_on_lost_comment=false
fn foo() {
    40 + 2;
    !40 + 2;
    40 + 2 == 42;

    40 /*test*/ + 2 == 42;
    40 + 2 /*test*/ == 42;
}

fn big() {
    assert(
        bjj_affine.contains(bjj_affine.gen)
            & bjj_affine.contains(p1_affine)
            & bjj_affine.contains(p2_affine)
            & bjj_affine.contains(p3_affine)
            & bjj_affine.contains(p4_affine)
            & bjj_affine.contains(p5_affine),
    );
}
