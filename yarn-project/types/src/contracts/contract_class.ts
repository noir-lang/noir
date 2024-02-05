import { FunctionSelector } from '@aztec/foundation/abi';
import { Fr } from '@aztec/foundation/fields';
import { PartialBy } from '@aztec/foundation/types';

const VERSION = 1 as const;

/**
 * A Contract Class in the protocol. Aztec differentiates contracts classes and instances, where a
 * contract class represents the code of the contract, but holds no state. Classes are identified by
 * an id that is a commitment to all its data.
 */
export interface ContractClass {
  /** Version of the contract class. */
  version: typeof VERSION;
  /** Hash of the contract artifact. The specification of this hash is not enforced by the protocol. Should include commitments to unconstrained code and compilation metadata. Intended to be used by clients to verify that an off-chain fetched artifact matches a registered class. */
  artifactHash: Fr;
  /** List of individual private functions, constructors included. */
  privateFunctions: PrivateFunction[];
  /** List of individual public functions. Should be removed once we switch to the AVM where all public bytecode is bundled together. */
  publicFunctions: PublicFunction[];
  /** Packed bytecode representation of the AVM bytecode for all public functions in this contract. Unused for now, see `publicFunctions`. */
  packedBytecode: Buffer;
}

/** Private function definition within a contract class. */
export interface PrivateFunction {
  /** Selector of the function. Calculated as the hash of the method name and parameters. The specification of this is not enforced by the protocol. */
  selector: FunctionSelector;
  /** Hash of the verification key associated to this private function. */
  vkHash: Fr;
  /**
   * Whether the function is internal.
   * @deprecated To be reimplemented as an app-level macro.
   */
  isInternal: boolean;
}

/** Public function definition within a contract class. */
export interface PublicFunction {
  /** Selector of the function. Calculated as the hash of the method name and parameters. The specification of this is not enforced by the protocol. */
  selector: FunctionSelector;
  /** Public bytecode. */
  bytecode: Buffer;
  /**
   * Whether the function is internal.
   * @deprecated To be reimplemented as an app-level macro.
   */
  isInternal: boolean;
}

/** Commitments to fields of a contract class. */
interface ContractClassCommitments {
  /** Identifier of the contract class. */
  id: Fr;
  /** Commitment to the public bytecode. */
  publicBytecodeCommitment: Fr;
  /** Root of the private functions tree  */
  privateFunctionsRoot: Fr;
}

/** A contract class with its precomputed id. */
export type ContractClassWithId = ContractClass & Pick<ContractClassCommitments, 'id'>;

/** A contract class with public bytecode information only. */
export type ContractClassPublic = PartialBy<ContractClass, 'privateFunctions'> &
  Pick<ContractClassCommitments, 'id' | 'privateFunctionsRoot'>;
