//! This module is adapted from the [Barustenberg][barustenberg] Rust implementation of the Barretenberg library.
//!
//! Code is used under the MIT license
//!
//! [barustenberg]: https://github.com/laudiacay/barustenberg/blob/df6bc6f095fe7f288bf6a12e7317fd8eb33d68ae/

use lazy_static::lazy_static;

use std::{collections::HashMap, fmt::Debug, sync::Mutex};

use ark_ec::{
    short_weierstrass::{Affine, SWCurveConfig},
    AffineRepr,
};

mod generators;
mod hash_to_curve;

pub(crate) use generators::derive_generators;

pub(crate) const DEFAULT_NUM_GENERATORS: usize = 8;
pub(crate) const DEFAULT_DOMAIN_SEPARATOR: &str = "DEFAULT_DOMAIN_SEPARATOR";

//Ref that can be imported to access pre-computed generators
lazy_static! {
    pub(crate) static ref GENERATOR_CONTEXT: Mutex<GeneratorContext<grumpkin::GrumpkinParameters>> =
        Mutex::new(GeneratorContext::default());
}

#[derive(Debug, Clone)]
pub(crate) struct GeneratorList<E: SWCurveConfig>(Vec<Affine<E>>);

// In barustenberg there exists a shared ladder storing cached precomputed values.
#[derive(Clone, Debug)]
pub(crate) struct GeneratorData<E: SWCurveConfig> {
    pub(crate) precomputed_generators: [Affine<E>; DEFAULT_NUM_GENERATORS],
    pub(crate) generator_map: HashMap<String, GeneratorList<E>>,
}

impl<E: SWCurveConfig> Default for GeneratorData<E> {
    fn default() -> Self {
        Self {
            precomputed_generators: Self::make_precomputed_generators(),
            generator_map: HashMap::new(),
        }
    }
}

impl<E: SWCurveConfig> GeneratorData<E> {
    fn make_precomputed_generators() -> [Affine<E>; DEFAULT_NUM_GENERATORS] {
        let mut output: [Affine<E>; DEFAULT_NUM_GENERATORS] =
            [Affine::zero(); DEFAULT_NUM_GENERATORS];
        let res: Vec<Affine<E>> = derive_generators(
            DEFAULT_DOMAIN_SEPARATOR.as_bytes(),
            DEFAULT_NUM_GENERATORS as u32,
            0,
        );
        output.copy_from_slice(&res[..DEFAULT_NUM_GENERATORS]);
        output
    }

    //NOTE: can add default arguments by wrapping function parameters with options
    pub(crate) fn get(
        &mut self,
        num_generators: usize,
        generator_offset: usize,
        domain_separator: &str,
    ) -> Vec<Affine<E>> {
        let is_default_domain = domain_separator == DEFAULT_DOMAIN_SEPARATOR;
        if is_default_domain && (num_generators + generator_offset) < DEFAULT_NUM_GENERATORS {
            return self.precomputed_generators.to_vec();
        }

        // Case 2: we want default generators, but more than we precomputed at compile time. If we have not yet copied
        // the default generators into the map, do so.
        if is_default_domain && !self.generator_map.is_empty() {
            let _ = self
                .generator_map
                .insert(
                    DEFAULT_DOMAIN_SEPARATOR.to_string(),
                    GeneratorList(self.precomputed_generators.to_vec()),
                )
                .unwrap();
        }

        //TODO: open to suggestions for this
        let mut generators = self.generator_map.get(DEFAULT_DOMAIN_SEPARATOR).unwrap().0.clone();

        if num_generators + generator_offset > generators.len() {
            let num_extra_generators = num_generators + generator_offset - generators.len();
            let extended_generators = derive_generators(
                domain_separator.as_bytes(),
                num_extra_generators as u32,
                generators.len() as u32,
            );

            generators.extend_from_slice(&extended_generators);
        }

        generators
    }
}

#[derive(Debug, Clone)]
pub(crate) struct GeneratorContext<E: SWCurveConfig> {
    pub(crate) offset: usize,
    pub(crate) domain_separator: &'static str,
    pub(crate) generators: GeneratorData<E>,
}

impl<E: SWCurveConfig> Default for GeneratorContext<E> {
    fn default() -> Self {
        Self {
            offset: 0,
            domain_separator: DEFAULT_DOMAIN_SEPARATOR,
            generators: GeneratorData::default(),
        }
    }
}
