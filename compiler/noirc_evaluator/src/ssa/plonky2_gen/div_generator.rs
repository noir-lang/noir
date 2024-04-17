use core::fmt::Debug;
use std::marker::PhantomData;

use super::config::{P2Builder, P2Field};
use plonky2::{
    field::types::{Field, PrimeField64},
    iop::{
        generator::{GeneratedValues, SimpleGenerator},
        witness::{PartitionWitness, Witness, WitnessWrite},
    },
    plonk::circuit_data::CommonCircuitData,
    util::serialization::IoResult,
};

use plonky2::iop::target::Target;

#[derive(Debug, Clone)]
struct VariableIntDivGenerator {
    numerator: Target,
    denominator: Target,
    pub quotient: Target,
    pub remainder: Target,
    _phantom: PhantomData<P2Field>,
}

impl VariableIntDivGenerator {
    pub fn new(builder: &mut P2Builder, numerator: Target, denominator: Target) -> Self {
        Self {
            numerator,
            denominator,
            quotient: builder.add_virtual_target(),
            remainder: builder.add_virtual_target(),
            _phantom: PhantomData,
        }
    }

    pub fn id() -> String {
        "VariableIntDivGenerator".to_string()
    }
}

impl SimpleGenerator<P2Field, 2> for VariableIntDivGenerator {
    fn id(&self) -> String {
        Self::id()
    }

    fn dependencies(&self) -> Vec<Target> {
        vec![self.numerator, self.denominator]
    }

    #[allow(unused_variables)]
    fn serialize(
        &self,
        dst: &mut Vec<u8>,
        _common_data: &CommonCircuitData<P2Field, 2>,
    ) -> IoResult<()> {
        todo!()
    }

    #[allow(unused_variables)]
    fn deserialize(
        src: &mut plonky2::util::serialization::Buffer,
        _common_data: &CommonCircuitData<P2Field, 2>,
    ) -> IoResult<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    fn run_once(
        &self,
        witness: &PartitionWitness<P2Field>,
        out_buffer: &mut GeneratedValues<P2Field>,
    ) {
        let numerator = witness.get_target(self.numerator).to_canonical_u64();
        let denominator = witness.get_target(self.denominator).to_canonical_u64();

        let quotient = numerator / denominator;
        let remainder = numerator % denominator;

        out_buffer.set_target(self.quotient, P2Field::from_canonical_u64(quotient));
        out_buffer.set_target(self.remainder, P2Field::from_canonical_u64(remainder));
    }
}

/// Add a whole number division operation to a circuit, returning the quotient and remainder.
/// This uses a custom `SimpleGenerator` internally, which will have performance implications.
pub fn add_div(
    builder: &mut P2Builder,
    numerator: Target,
    denominator: Target,
) -> (Target, Target) {
    let generator = VariableIntDivGenerator::new(builder, numerator, denominator);
    builder.add_simple_generator(generator.clone());

    let q_times_d = builder.mul(generator.quotient, denominator);
    let q_times_d_plus_r = builder.add(q_times_d, generator.remainder);
    let sanity = builder.is_equal(numerator, q_times_d_plus_r);
    builder.assert_bool(sanity);

    let z = builder.zero();
    let d_is_zero = builder.is_equal(denominator, z);
    let d_is_not_zero = builder.not(d_is_zero);
    builder.assert_bool(d_is_not_zero);

    (generator.quotient, generator.remainder)
}
