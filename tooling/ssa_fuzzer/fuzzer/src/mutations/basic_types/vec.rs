//! This file contains mechanisms for deterministically mutating a given vector
//! Types of mutations applied:
//! 1. Random (randomly select a new vector)
//! 2. Insert a random element at a random index
//! 3. Delete a random element at a random index
//! 4. Swap two random elements at random indices
//! 5. Mutate a random element at a random index
//! 6. Push a default element to the end of the vector

use crate::mutations::configuration::{
    SIZE_OF_LARGE_ARBITRARY_BUFFER, SIZE_OF_SMALL_ARBITRARY_BUFFER, VecMutationConfig,
    VecMutationOptions,
};
use libfuzzer_sys::arbitrary::{Arbitrary, Unstructured};
use rand::{Rng, rngs::StdRng};

trait MutateVec<T>
where
    T: for<'a> Arbitrary<'a> + Default,
{
    fn mutate(
        rng: &mut StdRng,
        vec: &mut Vec<T>,
        mutate_element_function: impl Fn(&mut T, &mut StdRng),
    );
}

/// Mutate the entire vector, replacing it with a random vector
struct RandomMutation;
impl<T> MutateVec<T> for RandomMutation
where
    T: for<'a> Arbitrary<'a> + Default,
{
    fn mutate(
        rng: &mut StdRng,
        vec: &mut Vec<T>,
        _mutate_element_function: impl Fn(&mut T, &mut StdRng),
    ) {
        let mut bytes = [0u8; SIZE_OF_LARGE_ARBITRARY_BUFFER];
        rng.fill(&mut bytes);
        *vec = Unstructured::new(&bytes).arbitrary().unwrap();
    }
}

/// Insert a random element at a random index
struct RandomInsertion;
impl<T> MutateVec<T> for RandomInsertion
where
    T: for<'a> Arbitrary<'a> + Default,
{
    fn mutate(
        rng: &mut StdRng,
        vec: &mut Vec<T>,
        _mutate_element_function: impl Fn(&mut T, &mut StdRng),
    ) {
        let mut bytes = [0u8; SIZE_OF_SMALL_ARBITRARY_BUFFER];
        rng.fill(&mut bytes);
        let element = Unstructured::new(&bytes).arbitrary().unwrap();
        if !vec.is_empty() {
            let index = rng.gen_range(0..vec.len());
            vec.insert(index, element);
        }
    }
}

/// Delete a random element at a random index
struct RandomDeletion;
impl<T> MutateVec<T> for RandomDeletion
where
    T: for<'a> Arbitrary<'a> + Default,
{
    fn mutate(
        rng: &mut StdRng,
        vec: &mut Vec<T>,
        _mutate_element_function: impl Fn(&mut T, &mut StdRng),
    ) {
        if !vec.is_empty() {
            let index = rng.gen_range(0..vec.len());
            vec.remove(index);
        }
    }
}

/// Swap two random elements at random indices
struct RandomSwap;
impl<T> MutateVec<T> for RandomSwap
where
    T: for<'a> Arbitrary<'a> + Default,
{
    fn mutate(
        rng: &mut StdRng,
        vec: &mut Vec<T>,
        _mutate_element_function: impl Fn(&mut T, &mut StdRng),
    ) {
        if !vec.is_empty() {
            let index1 = rng.gen_range(0..vec.len());
            let index2 = rng.gen_range(0..vec.len());
            vec.swap(index1, index2);
        }
    }
}

/// Mutate a random element at a random index
struct RandomElementMutation;
impl<T> MutateVec<T> for RandomElementMutation
where
    T: for<'a> Arbitrary<'a> + Default,
{
    fn mutate(
        rng: &mut StdRng,
        vec: &mut Vec<T>,
        mutate_element_function: impl Fn(&mut T, &mut StdRng),
    ) {
        if !vec.is_empty() {
            let index = rng.gen_range(0..vec.len());
            mutate_element_function(&mut vec[index], rng);
        }
    }
}

/// Push a default element to the end of the vector
struct PushDefaultElement;
impl<T> MutateVec<T> for PushDefaultElement
where
    T: for<'a> Arbitrary<'a> + Default,
{
    fn mutate(
        _rng: &mut StdRng,
        vec: &mut Vec<T>,
        _mutate_element_function: impl Fn(&mut T, &mut StdRng),
    ) {
        vec.push(T::default());
    }
}

pub(crate) fn mutate_vec<T>(
    vec: &mut Vec<T>,
    rng: &mut StdRng,
    mutate_element_function: impl Fn(&mut T, &mut StdRng),
    config: VecMutationConfig,
) where
    T: for<'a> Arbitrary<'a> + Default,
{
    match config.select(rng) {
        VecMutationOptions::Random => RandomMutation::mutate(rng, vec, mutate_element_function),
        VecMutationOptions::Insertion => RandomInsertion::mutate(rng, vec, mutate_element_function),
        VecMutationOptions::Deletion => RandomDeletion::mutate(rng, vec, mutate_element_function),
        VecMutationOptions::Swap => RandomSwap::mutate(rng, vec, mutate_element_function),
        VecMutationOptions::ElementMutation => {
            RandomElementMutation::mutate(rng, vec, mutate_element_function);
        }
        VecMutationOptions::PushDefault => {
            PushDefaultElement::mutate(rng, vec, mutate_element_function);
        }
    }
}
