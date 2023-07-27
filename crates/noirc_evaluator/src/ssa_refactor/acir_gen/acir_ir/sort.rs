use crate::errors::ICEError;

use super::generated_acir::GeneratedAcir;
use acvm::acir::native_types::{Expression, Witness};

impl GeneratedAcir {
    // Generates gates for a sorting network
    // returns witness corresponding to the network configuration and the expressions corresponding to the network output
    // in_expr: inputs of the sorting network
    // if generate_witness is false, it uses the witness provided in bits instead of generating them
    // in both cases it returns the witness of the network configuration
    // if generate_witness is true, bits is ignored
    pub(crate) fn permutation_layer(
        &mut self,
        in_expr: &[Expression],
        bits: &[Witness],
        generate_witness: bool,
    ) -> Result<(Vec<Witness>, Vec<Expression>), ICEError> {
        let n = in_expr.len();
        if n == 1 {
            return Ok((Vec::new(), in_expr.to_vec()));
        }
        let n1 = n / 2;

        // witness for the input switches
        let mut conf = iter_extended::vecmap(0..n1, |i| {
            if generate_witness {
                self.next_witness_index()
            } else {
                bits[i]
            }
        });

        // compute expressions after the input switches
        // If inputs are a1,a2, and the switch value is c, then we compute expressions b1,b2 where
        // b1 = a1+q, b2 = a2-q, q = c(a2-a1)
        let mut in_sub1 = Vec::new();
        let mut in_sub2 = Vec::new();
        for i in 0..n1 {
            //q = c*(a2-a1);
            let intermediate = self.mul_with_witness(
                &Expression::from(conf[i]),
                &(&in_expr[2 * i + 1] - &in_expr[2 * i]),
            )?;
            //b1=a1+q
            in_sub1.push(&intermediate + &in_expr[2 * i]);
            //b2=a2-q
            in_sub2.push(&in_expr[2 * i + 1] - &intermediate);
        }
        if n % 2 == 1 {
            in_sub2.push(match in_expr.last() {
                Some(in_expr) => in_expr.clone(),
                None => return Err(ICEError::UmptyArray { location: self.current_location }),
            });
        }
        let mut out_expr = Vec::new();
        // compute results for the sub networks
        let bits1 = if generate_witness { bits } else { &bits[n1 + (n - 1) / 2..] };
        let (w1, b1) = self.permutation_layer(&in_sub1, bits1, generate_witness)?;
        let bits2 = if generate_witness { bits } else { &bits[n1 + (n - 1) / 2 + w1.len()..] };
        let (w2, b2) = self.permutation_layer(&in_sub2, bits2, generate_witness)?;
        // apply the output switches
        for i in 0..(n - 1) / 2 {
            let c = if generate_witness { self.next_witness_index() } else { bits[n1 + i] };
            conf.push(c);
            let intermediate = self.mul_with_witness(&Expression::from(c), &(&b2[i] - &b1[i]))?;
            out_expr.push(&intermediate + &b1[i]);
            out_expr.push(&b2[i] - &intermediate);
        }
        if n % 2 == 0 {
            out_expr.push(match b1.last() {
                Some(b1) => b1.clone(),
                None => return Err(ICEError::UmptyArray { location: self.current_location }),
            });
        }
        out_expr.push(match b2.last() {
            Some(b2) => b2.clone(),
            None => return Err(ICEError::UmptyArray { location: self.current_location }),
        });
        conf.extend(w1);
        conf.extend(w2);
        Ok((conf, out_expr))
    }
}
