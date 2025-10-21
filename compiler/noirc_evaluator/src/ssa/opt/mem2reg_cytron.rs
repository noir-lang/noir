use rustc_hash::{FxHashMap, FxHashSet};

use crate::ssa::{ir::function::Function, ssa_gen::Ssa};


impl Ssa {
    pub(crate) fn mem2reg_simple(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.mem2reg_simple();
        }
        self
    }
}

impl Function {
    fn mem2reg_simple(&mut self) {
        let mut has_already = FxHashMap::default();
        let mut work = FxHashMap::default();
        let mut w = FxHashSet::default();
        let mut iter_count = 0;

        for v in self.variables() {
            iter_count += 1;

            for x in a(v) {
                work.insert(x, iter_count);
                w.insert(x);
            }

            while let Some(x) = w.pop() {
                for y in dominance_frontier(x) {
                    if has_already.get(y).unwrap_or(0) < iter_count {
                        place (V <- phi(V, ..., V)) at Y;
                        has_already.insert(y, iter_count);

                        if work.get(y).unwrap_or(0) < iter_count {
                            work.insert(y, iter_count);
                            w.insert(y);
                        }
                    }
                }
            }
        }
    }
}
