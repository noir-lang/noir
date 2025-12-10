//! This file contains mechanisms for deterministically mutating a given vector
//! Types of mutations applied:
//! 1. Random (randomly select a new vector)
//! 2. Insert a random element at a random index
//! 3. Delete a random element at a random index
//! 4. Swap two random elements at random indices
//! 5. Mutate a random element at a random index
//! 6. Push a default element to the end of the vector

use crate::mutations::configuration::{VecMutationConfig, VecMutationOptions};
use rand::{Rng, rngs::StdRng};

/// Insert a random element at a random index
struct RandomInsertion;
impl RandomInsertion {
    fn mutate<T>(
        rng: &mut StdRng,
        vec: &mut Vec<T>,
        generate_random_element_function: impl Fn(&mut StdRng) -> T,
    ) {
        let element = generate_random_element_function(rng);
        if !vec.is_empty() {
            let index = rng.gen_range(0..vec.len());
            vec.insert(index, element);
        } else {
            vec.push(element);
        }
    }
}

/// Delete a random element at a random index
struct RandomDeletion;
impl RandomDeletion {
    fn mutate<T>(rng: &mut StdRng, vec: &mut Vec<T>) {
        // do not remove the last element
        // because ssa fuzzer forbids empty arrays in initial witness
        // so we must keep at least one element
        if vec.len() > 1 {
            let index = rng.gen_range(0..vec.len());
            vec.remove(index);
        }
    }
}

/// Swap two random elements at random indices
struct RandomSwap;
impl RandomSwap {
    fn mutate<T>(rng: &mut StdRng, vec: &mut [T]) {
        if !vec.is_empty() {
            let index1 = rng.gen_range(0..vec.len());
            let index2 = rng.gen_range(0..vec.len());
            vec.swap(index1, index2);
        }
    }
}

/// Mutate a random element at a random index
struct RandomElementMutation;
impl RandomElementMutation {
    fn mutate<T>(
        rng: &mut StdRng,
        vec: &mut [T],
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
impl PushDefaultElement {
    fn mutate<T>(vec: &mut Vec<T>)
    where
        T: Default,
    {
        vec.push(T::default());
    }
}

pub(crate) fn mutate_vec<T>(
    vec: &mut Vec<T>,
    rng: &mut StdRng,
    mutate_element_function: impl Fn(&mut T, &mut StdRng),
    generate_random_element_function: impl Fn(&mut StdRng) -> T,
    config: VecMutationConfig,
) where
    T: Default,
{
    match config.select(rng) {
        VecMutationOptions::Insertion => {
            RandomInsertion::mutate(rng, vec, generate_random_element_function);
        }
        VecMutationOptions::Deletion => RandomDeletion::mutate(rng, vec),
        VecMutationOptions::Swap => RandomSwap::mutate(rng, vec),
        VecMutationOptions::ElementMutation => {
            RandomElementMutation::mutate(rng, vec, mutate_element_function);
        }
        VecMutationOptions::PushDefault => PushDefaultElement::mutate(vec),
    }
}
