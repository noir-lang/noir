#![allow(unused)]
//! Implements copy coalescing as detailed in https://cseweb.ucsd.edu/classes/sp02/cse231/kenpldi.pdf

use crate::{brillig::brillig_gen::{constant_allocation::ConstantAllocation, variable_liveness::VariableLiveness}, ssa::ir::{basic_block::BasicBlockId, dom::DominatorTree, function::Function, value::ValueId}};
use rustc_hash::{ FxHashSet as HashSet, FxHashMap as HashMap };



fn coalesce_copies(function: &Function, constants: &ConstantAllocation) -> VariableLiveness {
    /// Step 1: Build initial live ranges
    let liveness = VariableLiveness::from_function(function, constants);

    /// Step 2: Build the dominance forest
    let dom_tree = DominatorTree::with_function(function);
    let variables = todo!();
    let dom_forest = DominanceForest::new(dom_tree, variables);
}

struct DominanceForest {
    var_to_definition_block: HashMap<ValueId, BasicBlockId>,

    // edges represent collapsed dominator tree paths
}

impl DominanceForest {
    /// Step 2: Construct the dominance forest
    fn new(
        tree: DominatorTree,
        variables: HashSet<ValueId>,
    ) -> DominanceForest {
        for b in depth-first-order(tree) {
            preorder(b) = next preorder name;
            maxpreorder(b) = largest preorder number of b's descendents;
        }

        take S in dominator order;

        maxpreorder(VirtualRoot) = MAX;

        let mut current_parent = VirtualRoot;
        let mut stack = vec![VirtualRoot];

        for variable v in S in sorted order {
            while preorder(v) > maxpreorder(current_parent) {
                stack.pop();
                current_parent = stack.last().unwrap();
            }

            make v a child of current_parent;
            stack.push(v);
            current_parent = v;
        }
        remove VirtualRoot from DF;
    }

    /// Step 3: Walk the dominance forest
    fn walk(&self) {
        for depth-first traversal of DFi {
            if variable p is c's parent and is in the live-out set of c's defining block {
                if p cant interfere with any of its other children and c has fewer copies to insert than p {
                    insert copies for c and make c's children p's children
                } else {
                    insert copies for p
                }
            } else if parent p is in the live-in set of c's defining block or p and c have the same defining block {
                add the variable pair (p, c) to the list to check for local interference later
            }
        }
    }
}
