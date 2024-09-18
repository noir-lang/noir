use acvm::acir::brillig::MemoryAddress;
use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};
use im::Vector;

type MoveChain = Vec<(MemoryAddress, MemoryAddress)>;

pub(crate) struct MoveRegistersSolver {
    // The source addresses and their corresponding destinations
    movements: HashMap<MemoryAddress, MemoryAddress>,
    // The set of destination addresses
    destinations: HashSet<MemoryAddress>,
    // The source addresses that have been visited
    visited: HashSet<MemoryAddress>,
    // The chains of dependencies that we have found
    dependency_chain: Vec<Vector<MemoryAddress>>,
}

impl MoveRegistersSolver {
    pub(crate) fn sources_destinations_to_move_chains(
        sources: Vec<MemoryAddress>,
        destinations: Vec<MemoryAddress>,
    ) -> Vec<MoveChain> {
        assert_eq!(
            sources.len(),
            destinations.len(),
            "Different number of sources and destinations",
        );
        // Filter no-op movements
        let movements: Vec<_> = sources
            .into_iter()
            .zip(destinations.into_iter())
            .filter(|(source, destination)| source != destination)
            .collect();

        let total_movements = movements.len();
        let movements: HashMap<_, _> = movements.into_iter().collect();
        assert!(total_movements == movements.len(), "Multiple moves from the same register found");

        // We collect the sources again after filtering no-ops
        let sources = movements.keys().copied().collect();
        let destinations: HashSet<_> = movements.values().copied().collect();
        assert!(destinations.len() == total_movements, "Multiple moves to the same register found");

        let mut solver = MoveRegistersSolver {
            movements,
            visited: HashSet::default(),
            destinations,
            dependency_chain: Vec::new(),
        };

        solver.solve(sources);
        // Map the dependency chains to a chain of operations to perform
        solver
            .dependency_chain
            .into_iter()
            .map(|chain| {
                chain
                    .into_iter()
                    .map(|source| {
                        let destination = solver.movements.get(&source).unwrap();
                        (source, *destination)
                    })
                    .rev() // We reverse the chain to express what needs to be applied first
                    .collect()
            })
            .collect()
    }

    fn solve(&mut self, sources: Vec<MemoryAddress>) {
        // First, we'll find all the non-cyclic chains of movements.
        // All chains start with a source that is not written to
        let chain_heads: Vec<_> =
            sources.iter().filter(|source| !self.destinations.contains(source)).copied().collect();

        for source in chain_heads {
            self.explore(source, Vector::default());
        }

        // Then we handle the cycles
        for &source in sources.iter() {
            if self.visited.contains(&source) {
                continue;
            }
            self.explore(source, Vector::new());
        }
    }

    fn explore(
        &mut self,
        current_source: MemoryAddress,
        mut current_dependency_chain: Vector<MemoryAddress>,
    ) {
        // Record that we visited this source
        self.visited.insert(current_source);
        current_dependency_chain.push_back(current_source);

        // We need to check the target that this source is moving to
        let target = self.movements.get(&current_source).unwrap();
        let is_target_source = self.movements.contains_key(target);

        // Check if we are at the end of a chain or in a cycle
        if !is_target_source
            || current_dependency_chain
                .get(0)
                .and_then(|first: &MemoryAddress| Some(first == target))
                .unwrap_or(false)
        {
            self.dependency_chain.push(current_dependency_chain);
            return;
        }

        // Safety check that we are properly detecting cycles
        assert!(!self.visited.contains(target), "Movement cycle went undetected");
        // If the target is also a non visited source, then we need to explore it
        self.explore(*target, current_dependency_chain.clone());
    }
}

/// We have a loop if the destination of the first movement needs to be the source for the last
pub(crate) fn is_loop(chain: &MoveChain) -> bool {
    let first = chain.first().unwrap().1;
    let last = chain.last().unwrap().0;
    first == last
}

#[cfg(test)]
mod tests {
    use super::*;

    fn movements_to_source_and_destinations(
        movements: Vec<(usize, usize)>,
    ) -> (Vec<MemoryAddress>, Vec<MemoryAddress>) {
        let sources = movements.iter().map(|(source, _)| MemoryAddress::from(*source)).collect();
        let destinations =
            movements.iter().map(|(_, destination)| MemoryAddress::from(*destination)).collect();
        (sources, destinations)
    }

    fn create_move_chains(operations: Vec<Vec<(usize, usize)>>) -> Vec<MoveChain> {
        operations
            .into_iter()
            .map(|ops| {
                ops.into_iter()
                    .map(|(source, destination)| {
                        (MemoryAddress::from(source), MemoryAddress::from(destination))
                    })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn test_solver_simple_op() {
        let chains = MoveRegistersSolver::sources_destinations_to_move_chains(
            vec![MemoryAddress(1)],
            vec![MemoryAddress(5)],
        );
        assert_eq!(chains, create_move_chains(vec![vec![(1, 5)]]));
        assert!(!is_loop(&chains[0]));
    }

    #[test]
    fn test_solver_simple_loop() {
        let movements = vec![(4, 5), (1, 2), (3, 4), (2, 3), (5, 1)];
        let (sources, destinations) = movements_to_source_and_destinations(movements);
        let chains =
            MoveRegistersSolver::sources_destinations_to_move_chains(sources, destinations);
        assert_eq!(chains, create_move_chains(vec![vec![(4, 5), (3, 4), (2, 3), (1, 2), (5, 1)]]));
        assert!(is_loop(&chains[0]));
    }

    #[test]
    fn test_solver_simple_chain() {
        let movements = vec![(2, 3), (3, 4), (1, 2), (4, 5)];
        let (sources, destinations) = movements_to_source_and_destinations(movements);
        let chains =
            MoveRegistersSolver::sources_destinations_to_move_chains(sources, destinations);
        assert_eq!(chains, create_move_chains(vec![vec![(4, 5), (3, 4), (2, 3), (1, 2),]]));
        assert!(!is_loop(&chains[0]));
    }

    #[test]
    fn test_solver_chain_and_loop() {
        let movements = vec![(2, 3), (3, 4), (1, 5), (4, 2)];
        let (sources, destinations) = movements_to_source_and_destinations(movements);
        let chains =
            MoveRegistersSolver::sources_destinations_to_move_chains(sources, destinations);
        assert_eq!(chains, create_move_chains(vec![vec![(1, 5)], vec![(4, 2), (3, 4), (2, 3)]]));
        assert!(is_loop(&chains[1]));
    }

    #[test]
    fn test_no_op() {
        let chains = MoveRegistersSolver::sources_destinations_to_move_chains(
            vec![MemoryAddress(1)],
            vec![MemoryAddress(1)],
        );
        assert!(chains.is_empty());
    }

    #[test]
    #[should_panic(expected = "Multiple moves from the same register found")]
    fn test_multiple_destinations() {
        let movements = vec![(1, 2), (1, 3)];
        let (sources, destinations) = movements_to_source_and_destinations(movements);
        MoveRegistersSolver::sources_destinations_to_move_chains(sources, destinations);
    }

    #[test]
    #[should_panic(expected = "Multiple moves to the same register found")]
    fn test_multiple_sources() {
        let movements = vec![(1, 2), (3, 2)];
        let (sources, destinations) = movements_to_source_and_destinations(movements);
        MoveRegistersSolver::sources_destinations_to_move_chains(sources, destinations);
    }
}
