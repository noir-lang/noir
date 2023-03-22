/**
 * A dependency entry of Nargo.toml.
 */
export interface Dependency {
  /**
   * Path to the dependency.
   */
  path?: string;
  /**
   * Git repository of the dependency.
   */
  git?: string;
}

/**
 * A circuit type.
 */
export interface CircuitType {
  /**
   * The actual type.
   */
  kind: any;
}

/**
 * A parameter to the circuit.
 */
export interface Parameter {
  /**
   * The name of the parameter.
   */
  name: string;
  /**
   * The type of the parameter.
   */
  type: CircuitType;
  /**
   * The visibility of the parameter.
   */
  visibility: 'private' | 'public';
}

/**
 * The representation of a compiled circuit.
 */
export interface CompiledCircuit {
  /**
   * The bytecode of the circuit.
   */
  circuit: Array<number>;
  /**
   * The Noir ABI of the circuit.
   */
  abi: {
    /**
     * The circuit  parameters.
     */
    parameters: Array<Parameter>;
    /**
     * The witness indices for the parameters.
     */
    param_witnesses: Record<string, Array<number>>;
    /**
     * The circuit return type.
     */
    return_type: CircuitType | null;
    /**
     * The witness indices for the return value.
     */
    return_witnesses: Array<number>;
  };
}
