# Root Rollup circuit

### Circuit Description

This circuit rolls up other rollup proofs.

It is defined by a parameter `rollup_num`, of inner rollups. Let's also denote $M:=$`rollup_num` for convenience.

### Circuit Inputs: Summary

The inputs for the root rollup circuit are:

$$ \text{Root Rollup Inputs} = (\text{Public Inputs}, \text{Private Inputs}) \in \mathbb{F}_p^{27 + M * (12 * 32)} \times \mathbb{F}_p^{27M}$$

As previously, the field $\mathbb{F}_p$ is from the BN254 specification.

### Public Inputs 

The root rollup circuit contains `17` public inputs.

The first pubic input is a SHA256 hash (reduced modulo the BN254 group order) of the following parameters:

1. `rollup_id` (The location where `new_root_M` will be inserted in the roots tree)
1. `rollup_size`
1. `data_start_index`
1. `old_data_root`
1. `new_data_root`
1. `old_null_root`
1. `new_null_root`
1. `old_root_root`
1. `new_root_root`
1. `old_defi_root`
1. `new_defi_root`
1. `bridge_call_datas` (size is `NUM_BRIDGE_CALLS_PER_BLOCK`)
1. `defi_deposit_sums` (size is `NUM_BRIDGE_CALLS_PER_BLOCK`)
1. `encrypted_defi_interaction_notes` (size is `NUM_BRIDGE_CALLS_PER_BLOCK`)
1. `previous_defi_interaction_hash`
1. `rollup_benficiary`
1. For $i=1,..,M$
    1. The `public_inputs_hash` of the rollup

The remaining 16 public inputs are 68-bit limbs of two BN254 $\mathbb{G}_1$ elements. Each element is split into two $\mathbb{F}_q$ elements, which is in turn split into 4 68-bit limbs. 

The two $\mathbb{G}_1$ elements, $[P_1], [P_2]$, represent the `recursive_proof_output` - group elements that must satisfy the following pairing check in order for the set of recursive proofs in the root rollup circuit to be valid:

$e([P_1], [x]) == e([P_2], [1])$, where $[x]$ is the $\mathbb{G}_2$ element produced by the Ignition trusted setup ceremony.

### Broadcasted Inputs

In addition to the public inputs, the preimage to the above SHA256 hash is also broadcasted with the proof.

The purpose of the SHA256 compression is not to hide information, it is solely to reduce the number of public inputs to the circuit.

This is because, for a verifier smart contract on the Ethereum blockchain network, the computational cost of processing a public input is ~160 gas. The computational cost of including a 32-byte value in a SHA256 hash is 6 gas. Therefore reducing the public inputs via SHA256 hashing represents a significant gas saving, lowering the cost of processing a rollup block.

The `rollup_benficiary` is just added to the circuit to ensure the proof constructor can pay who they intend.

### Private Inputs

1. The recursive proof output of each inner rollup proof (4 $\mathbb{F}_q$ elements represented as 16 $\mathbb{F}_p$ elements, see above)
2. The remaining public inputs of each rollup proof

### Circuit Logic (Pseudocode)
1. For $i=2,..,M+1$, check that $Q_i = aggregate(PI_{i-1}, \pi_{i-1}, vk, Q_{i-1}, (i > 1))$
2. For $i=2,..,M$, check that `new_data_root`$_{i-1}$=`old_data_root`$_i$.
3. Validate `Update(old_data_roots_root, new_data_roots_root, rollup_id, new_data_root_M)`
4. Validate that the `new_defi_root` of each real inner rollup proof is equal to the input `new_defi_root` to the root rollup
5. Validate that the `bridge_call_datas` in each real inner rollup proof match the input `bridge_call_datas` to the root rollup
6. Accumulate defi deposits across inner rollup proofs
7. Add the input `defi_interaction_notes` in the `defi_tree` and compute `previous_defi_interaction_hash := Hash(defi_interaction_notes)`
8. Range constrain that `rollup_beneficiary` is an ethereum address,

where $vk$ is the verification key of the rollup circuit. 
