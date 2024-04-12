use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::{
        circuit_builder::CircuitBuilder, circuit_data::CircuitData,
        config::PoseidonGoldilocksConfig,
    },
};

const D: usize = 2;
pub(crate) type P2Field = GoldilocksField;
pub(crate) type P2Config = PoseidonGoldilocksConfig;
pub(crate) type P2CircuitData = CircuitData<P2Field, P2Config, D>;
pub(crate) type P2Builder = CircuitBuilder<P2Field, D>;
