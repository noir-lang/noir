use std::borrow::Borrow;

use plonky2::iop::target::{BoolTarget, Target};

use super::config::{P2Builder, P2Field};

pub trait AsmWriter {
    fn get_builder(&self) -> &P2Builder;
    fn get_mut_builder(&mut self) -> &mut P2Builder;
    fn move_builder(self) -> P2Builder;

    fn new(builder: P2Builder, show_plonky2: bool) -> Self;

    fn is_equal(&mut self, x: Target, y: Target) -> BoolTarget;
    fn zero(&mut self) -> Target;
    fn one(&mut self) -> Target;
    fn two(&mut self) -> Target;
    fn split_le(&mut self, integer: Target, num_bits: usize) -> Vec<BoolTarget>;
    fn _if(&mut self, b: BoolTarget, x: Target, y: Target) -> Target;
    fn exp_u64(&mut self, base: Target, exponent: u64) -> Target;
    fn constant(&mut self, c: P2Field) -> Target;
    fn constant_bool(&mut self, b: bool) -> BoolTarget;
    fn mul(&mut self, x: Target, y: Target) -> Target;
    fn and(&mut self, b1: BoolTarget, b2: BoolTarget) -> BoolTarget;
    fn or(&mut self, b1: BoolTarget, b2: BoolTarget) -> BoolTarget;
    fn add(&mut self, x: Target, y: Target) -> Target;
    fn sub(&mut self, x: Target, y: Target) -> Target;
    fn not(&mut self, b: BoolTarget) -> BoolTarget;
    fn assert_bool(&mut self, b: BoolTarget);
    fn connect(&mut self, x: Target, y: Target);
    fn register_public_inputs(&mut self, targets: &[Target]);
    fn add_many<T>(&mut self, terms: impl IntoIterator<Item = T> + Clone) -> Target
    where
        T: Borrow<Target>;
}
