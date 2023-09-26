/* tslint:disable */
/* eslint-disable */
/**
* Executes an ACIR circuit to generate the solved witness from the initial witness.
*
* @param {&WasmBlackBoxFunctionSolver} solver - A black box solver.
* @param {Uint8Array} circuit - A serialized representation of an ACIR circuit
* @param {WitnessMap} initial_witness - The initial witness map defining all of the inputs to `circuit`..
* @param {ForeignCallHandler} foreign_call_handler - A callback to process any foreign calls from the circuit.
* @returns {WitnessMap} The solved witness calculated by executing the circuit on the provided inputs.
*/
export function executeCircuitWithBlackBoxSolver(solver: WasmBlackBoxFunctionSolver, circuit: Uint8Array, initial_witness: WitnessMap, foreign_call_handler: ForeignCallHandler): Promise<WitnessMap>;
/**
* Executes an ACIR circuit to generate the solved witness from the initial witness.
*
* @param {Uint8Array} circuit - A serialized representation of an ACIR circuit
* @param {WitnessMap} initial_witness - The initial witness map defining all of the inputs to `circuit`..
* @param {ForeignCallHandler} foreign_call_handler - A callback to process any foreign calls from the circuit.
* @returns {WitnessMap} The solved witness calculated by executing the circuit on the provided inputs.
*/
export function executeCircuit(circuit: Uint8Array, initial_witness: WitnessMap, foreign_call_handler: ForeignCallHandler): Promise<WitnessMap>;
/**
* @returns {Promise<WasmBlackBoxFunctionSolver>}
*/
export function createBlackBoxSolver(): Promise<WasmBlackBoxFunctionSolver>;
/**
* Decompresses a compressed witness as outputted by Nargo into a `WitnessMap`.
*
* @param {Uint8Array} compressed_witness - A compressed witness.
* @returns {WitnessMap} The decompressed witness map.
*/
export function decompressWitness(compressed_witness: Uint8Array): WitnessMap;
/**
* Compresses a `WitnessMap` into the binary format outputted by Nargo.
*
* @param {Uint8Array} compressed_witness - A witness map.
* @returns {WitnessMap} A compressed witness map
*/
export function compressWitness(witness_map: WitnessMap): Uint8Array;
/**
* Returns the `BuildInfo` object containing information about how the installed package was built.
* @returns {BuildInfo} - Information on how the installed package was built.
*/
export function buildInfo(): BuildInfo;
/**
* Sets the package's logging level.
*
* @param {LogLevel} level - The maximum level of logging to be emitted.
*/
export function initLogLevel(level: LogLevel): void;
/**
* Extracts a `WitnessMap` containing the witness indices corresponding to the circuit's public inputs.
*
* @param {Uint8Array} circuit - A serialized representation of an ACIR circuit
* @param {WitnessMap} witness_map - The completed witness map after executing the circuit.
* @returns {WitnessMap} A witness map containing the circuit's public inputs.
* @param {Uint8Array} circuit
* @param {WitnessMap} solved_witness
* @returns {WitnessMap}
*/
export function getPublicWitness(circuit: Uint8Array, solved_witness: WitnessMap): WitnessMap;
/**
* Extracts a `WitnessMap` containing the witness indices corresponding to the circuit's public parameters.
*
* @param {Uint8Array} circuit - A serialized representation of an ACIR circuit
* @param {WitnessMap} witness_map - The completed witness map after executing the circuit.
* @returns {WitnessMap} A witness map containing the circuit's public parameters.
* @param {Uint8Array} circuit
* @param {WitnessMap} solved_witness
* @returns {WitnessMap}
*/
export function getPublicParametersWitness(circuit: Uint8Array, solved_witness: WitnessMap): WitnessMap;
/**
* Extracts a `WitnessMap` containing the witness indices corresponding to the circuit's return values.
*
* @param {Uint8Array} circuit - A serialized representation of an ACIR circuit
* @param {WitnessMap} witness_map - The completed witness map after executing the circuit.
* @returns {WitnessMap} A witness map containing the circuit's return values.
* @param {Uint8Array} circuit
* @param {WitnessMap} witness_map
* @returns {WitnessMap}
*/
export function getReturnWitness(circuit: Uint8Array, witness_map: WitnessMap): WitnessMap;

export type ForeignCallInput = string[]
export type ForeignCallOutput = string | string[]

/**
* A callback which performs an foreign call and returns the response.
* @callback ForeignCallHandler
* @param {string} name - The identifier for the type of foreign call being performed.
* @param {string[][]} inputs - An array of hex encoded inputs to the foreign call.
* @returns {Promise<string[]>} outputs - An array of hex encoded outputs containing the results of the foreign call.
*/
export type ForeignCallHandler = (name: string, inputs: ForeignCallInput[]) => Promise<ForeignCallOutput[]>;



/**
* @typedef {Object} BuildInfo - Information about how the installed package was built
* @property {string} gitHash - The hash of the git commit from which the package was built. 
* @property {string} version - The version of the package at the built git commit.
* @property {boolean} dirty - Whether the package contained uncommitted changes when built.
 */
export type BuildInfo = {
  gitHash: string;
  version: string;
  dirty: string;
}



export type ExecutionError = Error & {
    callStack?: string[];
};



export type LogLevel = "OFF" | "ERROR" | "WARN" | "INFO" | "DEBUG" | "TRACE";



// Map from witness index to hex string value of witness.
export type WitnessMap = Map<number, string>;


/**
* A struct representing a Trap
*/
export class Trap {
  free(): void;
/**
* @returns {Symbol}
*/
  static __wbgd_downcast_token(): Symbol;
}
/**
*/
export class WasmBlackBoxFunctionSolver {
  free(): void;
}
