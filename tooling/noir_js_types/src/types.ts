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

/** An id for a file. It's assigned during compilation. */
export type FileId = number;

/** Maps a file ID to its source code for debugging purposes. */
export type DebugFileMap = Record<
  FileId,
  {
    /** The source code of the file. */
    source: string;
    /** The path of the file. */
    path: string;
  }
>;

export type OpcodeLocation = string;

export type BrilligFunctionId = number;

/** A pointer to a specific section of the source code. */
export interface SourceCodeLocation {
  /** The section of the source code. */
  span: {
    /** The byte where the section starts. */
    start: number;
    /** The byte where the section ends. */
    end: number;
  };
  /** The source code file pointed to. */
  file: FileId;
}

export type OpcodeToLocationsMap = Record<OpcodeLocation, number>;

export type LocationNodeDebugInfo = {
  parent: number | null;
  value: SourceCodeLocation;
};

export type LocationTree = {
  locations: LocationNodeDebugInfo[];
};

/** The debug information for a given function. */
export interface DebugInfo {
  /** A map of the opcode location to the source code location. */
  location_tree: LocationTree;
  acir_locations: OpcodeToLocationsMap;
  /** For each Brillig function, we have a map of the opcode location to the source code location. */
  brillig_locations: Record<BrilligFunctionId, OpcodeToLocationsMap>;
}

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
  /** @description The debug information, compressed and base64 encoded. */
  debug_symbols: string;
  /**  @description The map of file ID to the source code and path of the file. */
  file_map: DebugFileMap;
};
