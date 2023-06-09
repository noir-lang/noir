// Type mappings for cbinds
// Can either export things directly or handle
// naming differences with the 'as' syntax
export {
  AggregationObject as NativeAggregationState,
  AztecAddress as Address,
  VerificationKey as VerificationKeyData,
  PrivateKernelPublicInputs as PublicInputs,
  Fr,
  Fq,
  G1AffineElement,
  NewContractData,
  FunctionData,
  OptionallyRevealedData,
  PublicDataRead,
  PublicDataUpdateRequest,
  CombinedAccumulatedData,
  PrivateHistoricTreeRoots,
  CombinedHistoricTreeRoots,
  ContractDeploymentData,
  TxContext,
  CombinedConstantData,
  KernelCircuitPublicInputs,
  Proof,
  PreviousKernelData,
  CallContext,
  ContractStorageUpdateRequest,
  ContractStorageRead,
  PublicCircuitPublicInputs,
  PublicCallStackItem,
  PublicCallData,
  PublicKernelInputs,
  CircuitError,
} from '../structs/index.js';

/**
 * A pointer to a Prover object in WebAssembly memory.
 */
export type RawPointerProverBase = number & {
  /**
   * A unique brand for distinguishing ProverBasePtr type objects.
   */
  __RawPointerProverBaseBrand: any;
};

/**
 * Type representing raw error messages returned by circuits.
 */
type RawError = {
  /**
   * An error code representing the specific issue encountered by the circuit.
   */
  code: number;
  /**
   * A descriptive error message providing details about the encountered issue in the circuit.
   */
  message: string;
};

/**
 * Distinguisher function for union types.
 * @param v - the distinguished type.
 */
export function isCircuitError(v: any): v is RawError {
  return v.code !== undefined && v.message !== undefined;
}
