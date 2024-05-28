use acir_field::AcirField;

use crate::native_types::Witness;
use std::cmp::Ordering;

use super::Expression;

// TODO: It's undecided whether `Expression` should implement `Ord/PartialOrd`.
// This is currently used in ACVM in the compiler.

impl<F: AcirField> Ord for Expression<F> {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut i1 = self.get_max_idx();
        let mut i2 = other.get_max_idx();
        let mut result = Ordering::Equal;
        while result == Ordering::Equal {
            let m1 = self.get_max_term(&mut i1);
            let m2 = other.get_max_term(&mut i2);
            if m1.is_none() && m2.is_none() {
                return Ordering::Equal;
            }
            result = Expression::<F>::cmp_max(m1, m2);
        }
        result
    }
}

impl<F: AcirField> PartialOrd for Expression<F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct WitnessIdx {
    linear: usize,
    mul: usize,
    second_term: bool,
}

impl<F: AcirField> Expression<F> {
    fn get_max_idx(&self) -> WitnessIdx {
        WitnessIdx {
            linear: self.linear_combinations.len(),
            mul: self.mul_terms.len(),
            second_term: true,
        }
    }

    /// Returns the maximum witness at the provided position, and decrement the position.
    ///
    /// This function assumes the gate is sorted
    fn get_max_term(&self, idx: &mut WitnessIdx) -> Option<Witness> {
        if idx.linear > 0 {
            if idx.mul > 0 {
                let mul_term = if idx.second_term {
                    self.mul_terms[idx.mul - 1].2
                } else {
                    self.mul_terms[idx.mul - 1].1
                };
                if self.linear_combinations[idx.linear - 1].1 > mul_term {
                    idx.linear -= 1;
                    Some(self.linear_combinations[idx.linear].1)
                } else {
                    if idx.second_term {
                        idx.second_term = false;
                    } else {
                        idx.mul -= 1;
                    }
                    Some(mul_term)
                }
            } else {
                idx.linear -= 1;
                Some(self.linear_combinations[idx.linear].1)
            }
        } else if idx.mul > 0 {
            if idx.second_term {
                idx.second_term = false;
                Some(self.mul_terms[idx.mul - 1].2)
            } else {
                idx.mul -= 1;
                Some(self.mul_terms[idx.mul].1)
            }
        } else {
            None
        }
    }

    fn cmp_max(m1: Option<Witness>, m2: Option<Witness>) -> Ordering {
        if let Some(m1) = m1 {
            if let Some(m2) = m2 {
                m1.cmp(&m2)
            } else {
                Ordering::Greater
            }
        } else if m2.is_some() {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}
