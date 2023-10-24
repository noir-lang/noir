import * as acvm from '@noir-lang/acvm_js';
import * as abi from '@noir-lang/noirc_abi';
import { CompiledCircuit, ProofData, Backend } from '@noir-lang/types';

// typedoc exports
/** @interface */
export { CompiledCircuit, ProofData, Backend };

export { Noir } from './program.js';

/** @ignore */
export { acvm, abi };
export { WitnessMap } from '@noir-lang/acvm_js';
