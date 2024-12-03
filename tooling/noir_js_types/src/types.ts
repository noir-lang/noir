export type Field = string | number | boolean;
export type InputValue = Field | InputMap | InputValue[];
export type InputMap = { [key: string]: InputValue };

export type Visibility = 'public' | 'private' | 'databus';
export type Sign = 'unsigned' | 'signed';
export type AbiType =
  | { kind: 'field' }
  | { kind: 'boolean' }
  | { kind: 'string'; length: number }
  | { kind: 'integer'; sign: Sign; width: number }
  | { kind: 'array'; length: number; type: AbiType }
  | { kind: 'tuple'; fields: AbiType[] }
  | { kind: 'struct'; path: string; fields: { name: string; type: AbiType }[] };

export type AbiParameter = {
  name: string;
  type: AbiType;
  visibility: Visibility;
};

export type AbiErrorType =
  | { error_kind: 'string'; string: string }
  | {
      error_kind: 'fmtstring';
      length: number;
      item_types: AbiType[];
    }
  | ({ error_kind: 'custom' } & AbiType);

// The payload for a raw assertion error returned on execution.
export type RawAssertionPayload = {
  selector: string;
  data: string[];
};

// Map from witness index to hex string value of witness.
export type WitnessMap = Map<number, string>;

export type Abi = {
  parameters: AbiParameter[];
  return_type: { abi_type: AbiType; visibility: Visibility } | null;
  error_types: Partial<Record<string, AbiErrorType>>;
};

export interface VerifierBackend {
  /**
   * @description Verifies a proof */
  verifyProof(proofData: ProofData): Promise<boolean>;

  /**
   * @description Destroys the backend */
  destroy(): Promise<void>;
}

export interface Backend extends VerifierBackend {
  /**
   * @description Generates a proof */
  generateProof(decompressedWitness: Uint8Array): Promise<ProofData>;

  /**
   *
   * @description Retrieves the artifacts from a proof in the Field format
   */
  generateRecursiveProofArtifacts(
    proofData: ProofData,
    numOfPublicInputs: number,
  ): Promise<{
    /** @description An array of Fields containing the proof */
    proofAsFields: string[];
    /** @description An array of Fields containing the verification key */
    vkAsFields: string[];
    /** @description A Field containing the verification key hash */
    vkHash: string;
  }>;
}

/**
 * @description
 * The representation of a proof
 * */
export type ProofData = {
  /** @description Public inputs of a proof */
  publicInputs: string[];
  /** @description An byte array representing the proof */
  proof: Uint8Array;
};

/**
 * @description
 * The representation of a compiled circuit
 * */
export type CompiledCircuit = {
  /** @description The bytecode of the circuit */
  bytecode: string;
  /** @description ABI representation of the circuit */
  abi: Abi;
};
