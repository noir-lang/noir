import * as acvm from '@noir-lang/acvm_js';
import * as abi from '@noir-lang/noirc_abi';
import { CompiledCircuit, ProofData } from '@noir-lang/types';

export {
  ecdsa_secp256r1_verify,
  ecdsa_secp256k1_verify,
  keccak256,
  blake2s256,
  sha256,
  xor,
  and,
} from '@noir-lang/acvm_js';

export { WitnessMap, ForeignCallHandler, ForeignCallInput, ForeignCallOutput } from '@noir-lang/acvm_js';

export { Noir } from './program.js';

/** @ignore */
export { acvm, abi };

// type exports for typedoc
export { CompiledCircuit, ProofData };
