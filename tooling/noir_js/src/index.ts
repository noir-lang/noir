import * as acvm from '@noir-lang/acvm_js';
import * as abi from '@noir-lang/noirc_abi';

export { acvm, abi };

import { generateWitness } from './witness_generation.js';
import { acirToUint8Array, witnessMapToUint8Array } from './serialize.js';
export { acirToUint8Array, witnessMapToUint8Array, generateWitness };
