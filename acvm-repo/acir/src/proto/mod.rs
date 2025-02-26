mod conversion;

#[allow(unreachable_pub)]
pub(crate) mod acir {
    include!(concat!(env!("OUT_DIR"), "/noir.proto.acir.rs"));
}

#[allow(unreachable_pub)]
pub(crate) mod brillig {
    include!(concat!(env!("OUT_DIR"), "/noir.proto.brillig.rs"));
}

#[allow(unreachable_pub)]
pub(crate) mod program {
    include!(concat!(env!("OUT_DIR"), "/noir.proto.program.rs"));
}
