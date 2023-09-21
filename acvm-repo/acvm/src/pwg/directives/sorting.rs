use std::collections::{BTreeMap, BTreeSet};

use acir::FieldElement;

// A sorting network is a graph of connected switches
// It is defined recursively so here we only keep track of the outer layer of switches
struct SortingNetwork {
    n: usize,                                // size of the network
    x_inputs: Vec<FieldElement>,             // inputs of the network
    y_inputs: Vec<FieldElement>,             // outputs of the network
    x_values: BTreeMap<FieldElement, usize>, // map for matching a y value with a x value
    y_values: BTreeMap<FieldElement, usize>, // map for matching a x value with a y value
    inner_x: Vec<FieldElement>,              // positions after the switch_x
    inner_y: Vec<FieldElement>, // positions after the sub-networks, and before the switch_y
    switch_x: Vec<bool>,        // outer switches for the inputs
    switch_y: Vec<bool>,        // outer switches for the outputs
    free: BTreeSet<usize>,      // outer switches available for looping
}

impl SortingNetwork {
    fn new(n: usize) -> SortingNetwork {
        let free_len = (n - 1) / 2;
        let mut free = BTreeSet::new();
        for i in 0..free_len {
            free.insert(i);
        }
        SortingNetwork {
            n,
            x_inputs: Vec::with_capacity(n),
            y_inputs: Vec::with_capacity(n),
            x_values: BTreeMap::new(),
            y_values: BTreeMap::new(),
            inner_x: Vec::with_capacity(n),
            inner_y: Vec::with_capacity(n),
            switch_x: Vec::with_capacity(n / 2),
            switch_y: Vec::with_capacity(free_len),
            free,
        }
    }

    fn init(&mut self, inputs: Vec<FieldElement>, outputs: Vec<FieldElement>) {
        let n = self.n;
        assert_eq!(inputs.len(), outputs.len());
        assert_eq!(inputs.len(), n);

        self.x_inputs = inputs;
        self.y_inputs = outputs;
        for i in 0..self.n {
            self.x_values.insert(self.x_inputs[i], i);
            self.y_values.insert(self.y_inputs[i], i);
        }
        self.switch_x = vec![false; n / 2];
        self.switch_y = vec![false; (n - 1) / 2];
        self.inner_x = vec![FieldElement::zero(); n];
        self.inner_y = vec![FieldElement::zero(); n];

        //Route the single wires so we do not need to handle this case later on
        self.inner_y[n - 1] = self.y_inputs[n - 1];
        if n % 2 == 0 {
            self.inner_y[n / 2 - 1] = self.y_inputs[n - 2];
        } else {
            self.inner_x[n - 1] = self.x_inputs[n - 1];
        }
    }

    //route a wire from outputs to its value in the inputs
    fn route_out_wire(&mut self, y: usize, sub: bool) -> usize {
        // sub <- y
        if self.is_single_y(y) {
            assert!(sub);
        } else {
            let port = y % 2 != 0;
            let s1 = sub ^ port;
            let inner = self.compute_inner(y, s1);
            self.configure_y(y, s1, inner);
        }
        // x <- sub
        let x = self.x_values.remove(&self.y_inputs[y]).unwrap();
        if !self.is_single_x(x) {
            let port2 = x % 2 != 0;
            let s2 = sub ^ port2;
            let inner = self.compute_inner(x, s2);
            self.configure_x(x, s2, inner);
        }
        x
    }

    //route a wire from inputs to its value in the outputs
    fn route_in_wire(&mut self, x: usize, sub: bool) -> usize {
        // x -> sub
        assert!(!self.is_single_x(x));
        let port = x % 2 != 0;
        let s1 = sub ^ port;
        let inner = self.compute_inner(x, s1);
        self.configure_x(x, s1, inner);

        // sub -> y
        let y = self.y_values.remove(&self.x_inputs[x]).unwrap();
        if !self.is_single_y(y) {
            let port = y % 2 != 0;
            let s2 = sub ^ port;
            let inner = self.compute_inner(y, s2);
            self.configure_y(y, s2, inner);
        }
        y
    }

    //update the computed switch and inner values for an input wire
    fn configure_x(&mut self, x: usize, switch: bool, inner: usize) {
        self.inner_x[inner] = self.x_inputs[x];
        self.switch_x[x / 2] = switch;
    }

    //update the computed switch and inner values for an output wire
    fn configure_y(&mut self, y: usize, switch: bool, inner: usize) {
        self.inner_y[inner] = self.y_inputs[y];
        self.switch_y[y / 2] = switch;
    }

    // returns the other wire belonging to the same switch
    fn sibling(index: usize) -> usize {
        index + 1 - 2 * (index % 2)
    }

    // returns a free switch
    fn take(&mut self) -> Option<usize> {
        self.free.first().copied()
    }

    fn is_single_x(&self, a: usize) -> bool {
        let n = self.x_inputs.len();
        n % 2 == 1 && a == n - 1
    }

    fn is_single_y(&mut self, a: usize) -> bool {
        let n = self.x_inputs.len();
        a >= n - 2 + n % 2
    }

    // compute the inner position of idx through its switch
    fn compute_inner(&self, idx: usize, switch: bool) -> usize {
        if switch ^ (idx % 2 == 1) {
            idx / 2 + self.n / 2
        } else {
            idx / 2
        }
    }

    fn new_start(&mut self) -> (Option<usize>, usize) {
        let next = self.take();
        if let Some(switch) = next {
            (next, 2 * switch)
        } else {
            (None, 0)
        }
    }
}

// Computes the control bits of the sorting network which transform inputs into outputs
// implementation is based on https://www.mdpi.com/2227-7080/10/1/16
pub(super) fn route(inputs: Vec<FieldElement>, outputs: Vec<FieldElement>) -> Vec<bool> {
    assert_eq!(inputs.len(), outputs.len());
    match inputs.len() {
        0 => Vec::new(),
        1 => {
            assert_eq!(inputs[0], outputs[0]);
            Vec::new()
        }
        2 => {
            if inputs[0] == outputs[0] {
                assert_eq!(inputs[1], outputs[1]);
                vec![false]
            } else {
                assert_eq!(inputs[1], outputs[0]);
                assert_eq!(inputs[0], outputs[1]);
                vec![true]
            }
        }
        _ => {
            let n = inputs.len();

            let mut result;
            let n1 = n / 2;
            let in_sub1;
            let out_sub1;
            let in_sub2;
            let out_sub2;

            // process the outer layer in a code block so that the intermediate data is cleared before recursion
            {
                let mut network = SortingNetwork::new(n);
                network.init(inputs, outputs);

                //We start with the last single wire
                let mut out_idx = n - 1;
                let mut start_sub = true; //it is connected to the lower inner network
                let mut switch = None;
                let mut start = None;

                while !network.free.is_empty() {
                    // the processed switch is no more available
                    if let Some(free_switch) = switch {
                        network.free.remove(&free_switch);
                    }

                    // connect the output wire to its matching input
                    let in_idx = network.route_out_wire(out_idx, start_sub);
                    if network.is_single_x(in_idx) {
                        start_sub = !start_sub; //We need to restart, but did not complete the loop so we switch the sub network
                        (start, out_idx) = network.new_start();
                        switch = start;
                        continue;
                    }

                    // loop from the sibling
                    let next = SortingNetwork::sibling(in_idx);
                    // connect the input wire to its matching output, using the other sub-network
                    out_idx = network.route_in_wire(next, !start_sub);
                    switch = Some(out_idx / 2);
                    if start == switch || network.is_single_y(out_idx) {
                        //loop is complete, need a fresh start
                        (start, out_idx) = network.new_start();
                        switch = start;
                    } else {
                        // we loop back from the sibling
                        out_idx = SortingNetwork::sibling(out_idx);
                    }
                }
                //All the wires are connected, we can now route the sub-networks
                result = network.switch_x;
                result.extend(network.switch_y);
                in_sub1 = network.inner_x[0..n1].to_vec();
                in_sub2 = network.inner_x[n1..].to_vec();
                out_sub1 = network.inner_y[0..n1].to_vec();
                out_sub2 = network.inner_y[n1..].to_vec();
            }
            let s1 = route(in_sub1, out_sub1);
            result.extend(s1);
            let s2 = route(in_sub2, out_sub2);
            result.extend(s2);
            result
        }
    }
}

#[cfg(test)]
mod tests {
    // Silence `unused_crate_dependencies` warning
    use paste as _;
    use proptest as _;

    use super::route;
    use acir::FieldElement;
    use rand::prelude::*;

    fn execute_network(config: Vec<bool>, inputs: Vec<FieldElement>) -> Vec<FieldElement> {
        let n = inputs.len();
        if n == 1 {
            return inputs;
        }
        let mut in1 = Vec::new();
        let mut in2 = Vec::new();
        //layer 1:
        for i in 0..n / 2 {
            if config[i] {
                in1.push(inputs[2 * i + 1]);
                in2.push(inputs[2 * i]);
            } else {
                in1.push(inputs[2 * i]);
                in2.push(inputs[2 * i + 1]);
            }
        }
        if n % 2 == 1 {
            in2.push(*inputs.last().unwrap());
        }
        let n2 = n / 2 + (n - 1) / 2;
        let n3 = n2 + switch_nb(n / 2);
        let mut result = Vec::new();
        let out1 = execute_network(config[n2..n3].to_vec(), in1);
        let out2 = execute_network(config[n3..].to_vec(), in2);
        //last layer:
        for i in 0..(n - 1) / 2 {
            if config[n / 2 + i] {
                result.push(out2[i]);
                result.push(out1[i]);
            } else {
                result.push(out1[i]);
                result.push(out2[i]);
            }
        }
        if n % 2 == 0 {
            result.push(*out1.last().unwrap());
            result.push(*out2.last().unwrap());
        } else {
            result.push(*out2.last().unwrap())
        }
        result
    }

    // returns the number of switches in the network
    fn switch_nb(n: usize) -> usize {
        let mut s = 0;
        for i in 0..n {
            s += f64::from((i + 1) as u32).log2().ceil() as usize;
        }
        s
    }

    #[test]
    fn test_route() {
        //basic tests
        let a = vec![
            FieldElement::from(1_i128),
            FieldElement::from(2_i128),
            FieldElement::from(3_i128),
        ];
        let b = vec![
            FieldElement::from(1_i128),
            FieldElement::from(2_i128),
            FieldElement::from(3_i128),
        ];
        let c = route(a, b);
        assert_eq!(c, vec![false, false, false]);

        let a = vec![
            FieldElement::from(1_i128),
            FieldElement::from(2_i128),
            FieldElement::from(3_i128),
        ];
        let b = vec![
            FieldElement::from(1_i128),
            FieldElement::from(3_i128),
            FieldElement::from(2_i128),
        ];
        let c = route(a, b);
        assert_eq!(c, vec![false, false, true]);

        let a = vec![
            FieldElement::from(1_i128),
            FieldElement::from(2_i128),
            FieldElement::from(3_i128),
        ];
        let b = vec![
            FieldElement::from(3_i128),
            FieldElement::from(2_i128),
            FieldElement::from(1_i128),
        ];
        let c = route(a, b);
        assert_eq!(c, vec![true, true, true]);

        let a = vec![
            FieldElement::from(0_i128),
            FieldElement::from(1_i128),
            FieldElement::from(2_i128),
            FieldElement::from(3_i128),
        ];
        let b = vec![
            FieldElement::from(2_i128),
            FieldElement::from(3_i128),
            FieldElement::from(0_i128),
            FieldElement::from(1_i128),
        ];
        let c = route(a, b);
        assert_eq!(c, vec![false, true, true, true, true]);

        let a = vec![
            FieldElement::from(0_i128),
            FieldElement::from(1_i128),
            FieldElement::from(2_i128),
            FieldElement::from(3_i128),
            FieldElement::from(4_i128),
        ];
        let b = vec![
            FieldElement::from(0_i128),
            FieldElement::from(3_i128),
            FieldElement::from(4_i128),
            FieldElement::from(2_i128),
            FieldElement::from(1_i128),
        ];
        let c = route(a, b);
        assert_eq!(c, vec![false, false, false, true, false, true, false, true]);

        // random tests
        for i in 2..50 {
            let mut a = vec![FieldElement::zero()];
            for j in 0..i - 1 {
                a.push(a[j] + FieldElement::one());
            }
            let mut rng = rand::thread_rng();
            let mut b = a.clone();
            b.shuffle(&mut rng);
            let c = route(a.clone(), b.clone());
            assert_eq!(b, execute_network(c, a));
        }
    }
}
