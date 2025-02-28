pub(crate) mod convert;

pub(crate) mod acir {

    #[allow(unreachable_pub)]
    pub(crate) mod native {
        include!(concat!(env!("OUT_DIR"), "/acvm.acir.native.rs"));
    }

    #[allow(unreachable_pub)]
    pub(crate) mod witness {
        include!(concat!(env!("OUT_DIR"), "/acvm.acir.witness.rs"));
    }

    #[allow(unreachable_pub)]
    pub(crate) mod circuit {
        include!(concat!(env!("OUT_DIR"), "/acvm.acir.circuit.rs"));
    }
}

#[allow(unreachable_pub, clippy::enum_variant_names)]
pub(crate) mod brillig {
    include!(concat!(env!("OUT_DIR"), "/acvm.brillig.rs"));
}

#[allow(unreachable_pub)]
pub(crate) mod program {
    include!(concat!(env!("OUT_DIR"), "/acvm.program.rs"));
}
