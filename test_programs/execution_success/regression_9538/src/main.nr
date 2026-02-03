unconstrained fn main(a: bool, mut b: (str<3>, str<3>)) -> pub str<3> {
    if a {
        func_2((if a { (&mut b.1) } else { (&mut b.0) }))
    } else {
        func_2((if a { (&mut b.1) } else { (&mut b.0) }))
    }
}
unconstrained fn func_2(a: &mut str<3>) -> str<3> {
    *a
}
