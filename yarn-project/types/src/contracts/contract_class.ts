import { type FunctionSelector } from '@aztec/foundation/abi';
import { type Fr } from '@aztec/foundation/fields';

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
}

/** Public function definition within a contract class. */
export interface PublicFunction {
  /** Selector of the function. Calculated as the hash of the method name and parameters. The specification of this is not enforced by the protocol. */
  selector: FunctionSelector;
  /** Public bytecode. */
  bytecode: Buffer;
}

/** Unconstrained function definition. */
export interface UnconstrainedFunction {
  /** Selector of the function. Calculated as the hash of the method name and parameters. The specification of this is not enforced by the protocol. */
  selector: FunctionSelector;
  /** Brillig. */
  bytecode: Buffer;
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

/** A contract class with public bytecode information, and optional private and unconstrained. */
export type ContractClassPublic = {
  privateFunctions: ExecutablePrivateFunctionWithMembershipProof[];
  unconstrainedFunctions: UnconstrainedFunctionWithMembershipProof[];
} & Pick<ContractClassCommitments, 'id' | 'privateFunctionsRoot'> &
  Omit<ContractClass, 'privateFunctions'>;

/** Private function definition with executable bytecode. */
export interface ExecutablePrivateFunction extends PrivateFunction {
  /** ACIR and Brillig bytecode */
  bytecode: Buffer;
}

/** Sibling paths and sibling commitments for proving membership of a private function within a contract class. */
export type PrivateFunctionMembershipProof = {
  artifactMetadataHash: Fr;
  functionMetadataHash: Fr;
  unconstrainedFunctionsArtifactTreeRoot: Fr;
  privateFunctionTreeSiblingPath: Fr[];
  privateFunctionTreeLeafIndex: number;
  artifactTreeSiblingPath: Fr[];
  artifactTreeLeafIndex: number;
};

/** A private function with a memebership proof. */
export type ExecutablePrivateFunctionWithMembershipProof = ExecutablePrivateFunction & PrivateFunctionMembershipProof;

/** Sibling paths and commitments for proving membership of an unconstrained function within a contract class. */
export type UnconstrainedFunctionMembershipProof = {
  artifactMetadataHash: Fr;
  functionMetadataHash: Fr;
  privateFunctionsArtifactTreeRoot: Fr;
  artifactTreeSiblingPath: Fr[];
  artifactTreeLeafIndex: number;
};

/** An unconstrained function with a membership proof. */
export type UnconstrainedFunctionWithMembershipProof = UnconstrainedFunction & UnconstrainedFunctionMembershipProof;
