//! This file contains mechanisms for deterministically mutating a given vector
//! Types of mutations applied:
//! 1. Random (randomly select a new vector)
//! 2. Insert a random element at a random index
//! 3. Delete a random element at a random index
//! 4. Swap two random elements at random indices
//! 5. Mutate a random element at a random index
//! 6. Push a default element to the end of the vector

use crate::mutations::configuration::{
    SIZE_OF_LARGE_ARBITRARY_BUFFER, VecMutationConfig, VecMutationOptions,
};
use libfuzzer_sys::arbitrary::{Arbitrary, Unstructured};
use rand::{Rng, rngs::StdRng};

/// Mutate the entire vector, replacing it with a random vector
struct RandomMutation;
impl RandomMutation {
    fn mutate<T>(rng: &mut StdRng, vec: &mut Vec<T>)
    where
        T: for<'a> Arbitrary<'a>,
    {
        let mut bytes = [0u8; SIZE_OF_LARGE_ARBITRARY_BUFFER];
        rng.fill(&mut bytes);
        *vec = Unstructured::new(&bytes).arbitrary().unwrap();
    }
}

/// Insert a random element at a random index
struct RandomInsertion;
impl RandomInsertion {
    fn mutate<T>(
        rng: &mut StdRng,
        vec: &mut Vec<T>,
        generate_random_element_function: impl Fn(&mut StdRng) -> T,
    ) where
        T: for<'a> Arbitrary<'a>,
    {
        let element = generate_random_element_function(rng);
        if !vec.is_empty() {
            let index = rng.random_range(0..vec.len());
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
        if !vec.is_empty() {
            let index = rng.random_range(0..vec.len());
            vec.remove(index);
        }
    }
}

/// Swap two random elements at random indices
struct RandomSwap;
impl RandomSwap {
    fn mutate<T>(rng: &mut StdRng, vec: &mut [T]) {
        if !vec.is_empty() {
            let index1 = rng.random_range(0..vec.len());
            let index2 = rng.random_range(0..vec.len());
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
            let index = rng.random_range(0..vec.len());
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
    T: for<'a> Arbitrary<'a> + Default,
{
    match config.select(rng) {
        VecMutationOptions::Random => RandomMutation::mutate(rng, vec),
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
