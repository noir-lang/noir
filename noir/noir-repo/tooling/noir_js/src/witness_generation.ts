import { abiDecodeError, abiEncode, InputMap } from '@noir-lang/noirc_abi';
import { base64Decode } from './base64_decode.js';
import {
  WitnessStack,
  ForeignCallHandler,
  ForeignCallInput,
  createBlackBoxSolver,
  WasmBlackBoxFunctionSolver,
  executeProgramWithBlackBoxSolver,
  ExecutionError,
} from '@noir-lang/acvm_js';
import { Abi, CompiledCircuit } from '@noir-lang/types';

let solver: Promise<WasmBlackBoxFunctionSolver>;

const getSolver = (): Promise<WasmBlackBoxFunctionSolver> => {
  if (!solver) {
    solver = createBlackBoxSolver();
  }
  return solver;
};

const defaultForeignCallHandler: ForeignCallHandler = async (name: string, args: ForeignCallInput[]) => {
  if (name == 'print') {
    // By default we do not print anything for `print` foreign calls due to a need for formatting,
    // however we provide an empty response in order to not halt execution.
    //
    // If a user needs to print values then they should provide a custom foreign call handler.
    return [];
  }
  throw Error(`Unexpected oracle during execution: ${name}(${args.join(', ')})`);
};

// Payload is any since it can be of any type defined by the circuit dev.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type ErrorWithPayload = ExecutionError & { decodedAssertionPayload?: any };

function parseErrorPayload(abi: Abi, originalError: ExecutionError): Error {
  const payload = originalError.rawAssertionPayload;
  if (!payload) return originalError;
  const enrichedError = originalError as ErrorWithPayload;

  try {
    // Decode the payload
    const decodedPayload = abiDecodeError(abi, payload);

    if (typeof decodedPayload === 'string') {
      // If it's a string, just add it to the error message
      enrichedError.message = `Circuit execution failed: ${decodedPayload}`;
    } else {
      // If not, attach the payload to the original error
      enrichedError.decodedAssertionPayload = decodedPayload;
    }
  } catch (_errorDecoding) {
    // Ignore errors decoding the payload
  }

  return enrichedError;
}

// Generates the witnesses needed to feed into the chosen proving system
export async function generateWitness(
  compiledProgram: CompiledCircuit,
  inputs: InputMap,
  foreignCallHandler: ForeignCallHandler = defaultForeignCallHandler,
): Promise<WitnessStack> {
  // Throws on ABI encoding error
  const witnessMap = abiEncode(compiledProgram.abi, inputs);

  // Execute the circuit to generate the rest of the witnesses and serialize
  // them into a Uint8Array.
  try {
    const solvedWitness = await executeProgramWithBlackBoxSolver(
      await getSolver(),
      base64Decode(compiledProgram.bytecode),
      witnessMap,
      foreignCallHandler,
    );
    return solvedWitness;
  } catch (err) {
    // Typescript types catched errors as unknown or any, so we need to narrow its type to check if it has raw assertion payload.
    if (typeof err === 'object' && err !== null && 'rawAssertionPayload' in err) {
      throw parseErrorPayload(compiledProgram.abi, err as ExecutionError);
    }
    throw new Error(`Circuit execution failed: ${err}`);
  }
}
