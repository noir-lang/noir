import { Buffer } from 'buffer';
import { CircuitsWasm } from '../wasm/index.js';
import {
  FunctionData,
  FUNCTION_SELECTOR_NUM_BYTES,
  ARGS_LENGTH,
  TxRequest,
  NewContractData,
  FunctionLeafPreimage,
  SignedTxRequest,
  PublicCallStackItem,
} from '../index.js';
import { serializeToBuffer, serializeBufferArrayToVector } from '../utils/serialize.js';
import { AsyncWasmWrapper, WasmWrapper } from '@aztec/foundation/wasm';
import { Fr } from '@aztec/foundation/fields';
import { AztecAddress } from '@aztec/foundation/aztec-address';

/**
 * Synchronously calls a wasm function.
 * @param wasm - The wasm wrapper.
 * @param fnName - The name of the function to call.
 * @param input - The input buffer or object serializable to a buffer.
 * @param expectedOutputLength - The expected length of the output buffer.
 * @returns The output buffer.
 */
export function wasmSyncCall(
  wasm: WasmWrapper,
  fnName: string,
  input:
    | Buffer
    | {
        /**
         * Signature of the target serialization function.
         */
        toBuffer: () => Buffer;
      },
  expectedOutputLength: number,
): Buffer {
  const inputData: Buffer = input instanceof Buffer ? input : input.toBuffer();
  const outputBuf = wasm.call('bbmalloc', expectedOutputLength);
  const inputBuf = wasm.call('bbmalloc', inputData.length);
  wasm.writeMemory(inputBuf, inputData);
  wasm.call(fnName, inputBuf, outputBuf);
  const buf = Buffer.from(wasm.getMemorySlice(outputBuf, outputBuf + expectedOutputLength));
  wasm.call('bbfree', outputBuf);
  wasm.call('bbfree', inputBuf);
  return buf;
}

/**
 * Asynchronously calls a wasm function. Required if the wasm call has a callback into an async js function.
 * @param wasm - The wasm wrapper.
 * @param fnName - The name of the function to call.
 * @param input - The input buffer or object serializable to a buffer.
 * @param expectedOutputLength - The expected length of the output buffer.
 * @returns The output buffer.
 */
export async function wasmAsyncCall(
  wasm: AsyncWasmWrapper,
  fnName: string,
  input:
    | Buffer
    | {
        /**
         * Signature of the target serialization function.
         */
        toBuffer: () => Buffer;
      },
  expectedOutputLength: number,
): Promise<Buffer> {
  const inputData: Buffer = input instanceof Buffer ? input : input.toBuffer();
  const outputBuf = wasm.call('bbmalloc', expectedOutputLength);
  const inputBuf = wasm.call('bbmalloc', inputData.length);
  wasm.writeMemory(inputBuf, inputData);
  await wasm.asyncCall(fnName, inputBuf, outputBuf);
  const buf = Buffer.from(wasm.getMemorySlice(outputBuf, outputBuf + expectedOutputLength));
  wasm.call('bbfree', outputBuf);
  wasm.call('bbfree', inputBuf);
  return buf;
}

/**
 * Writes input buffers to wasm memory, calls a wasm function, and returns the output buffer.
 * @param wasm - Circuits wasm.
 * @param fnName - The name of the function to call.
 * @param inputBuffers - Buffers to write to wasm memory.
 * @param expectedOutputLength - The expected length of the output buffer.
 * @returns The output buffer.
 */
export async function inputBuffersToOutputBuffer(
  wasm: CircuitsWasm,
  fnName: string,
  inputBuffers: Buffer[],
  expectedOutputLength: number,
) {
  const offsets: number[] = [];
  const totalLength = inputBuffers.reduce((total, cur) => {
    offsets.push(total);
    return total + cur.length;
  }, 0);

  const outputBuf = wasm.call('bbmalloc', expectedOutputLength);
  const inputBuf = wasm.call('bbmalloc', totalLength);
  wasm.writeMemory(inputBuf, Buffer.concat(inputBuffers));
  const args = offsets.map(offset => inputBuf + offset);
  await wasm.asyncCall(fnName, ...args, outputBuf);
  const output = Buffer.from(wasm.getMemorySlice(outputBuf, outputBuf + expectedOutputLength));
  wasm.call('bbfree', inputBuf);
  wasm.call('bbfree', outputBuf);
  return output;
}

/**
 * Computes a hash of a transaction request.
 * @param wasm - Circuits wasm.
 * @param txRequest - The transaction request.
 * @returns The hash of the transaction request.
 */
export async function hashTxRequest(wasm: CircuitsWasm, txRequest: TxRequest): Promise<Buffer> {
  wasm.call('pedersen__init');
  return await wasmAsyncCall(wasm, 'abis__hash_tx_request', txRequest, 32);
}

/**
 * Computes a function selector from a given function signature.
 * @param wasm - Circuits wasm.
 * @param funcSig - The function signature.
 * @returns The function selector.
 */
export async function computeFunctionSelector(wasm: CircuitsWasm, funcSig: string): Promise<Buffer> {
  return await wasmAsyncCall(
    wasm,
    'abis__compute_function_selector',
    Buffer.from(funcSig),
    FUNCTION_SELECTOR_NUM_BYTES,
  );
}

/**
 * Computes a hash of a given verification key.
 * @param wasm - Circuits wasm.
 * @param vkBuf - The verification key.
 * @returns The hash of the verification key.
 */
export async function hashVK(wasm: CircuitsWasm, vkBuf: Buffer) {
  wasm.call('pedersen__init');
  return await wasmAsyncCall(wasm, 'abis__hash_vk', vkBuf, 32);
}

/**
 * Computes a function leaf from a given preimage.
 * @param wasm - Circuits wasm.
 * @param fnLeaf - The function leaf preimage.
 * @returns The function leaf.
 */
export async function computeFunctionLeaf(wasm: CircuitsWasm, fnLeaf: FunctionLeafPreimage): Promise<Fr> {
  wasm.call('pedersen__init');
  return Fr.fromBuffer(await wasmAsyncCall(wasm, 'abis__compute_function_leaf', fnLeaf, 32));
}

/**
 * Computes a function tree root from function leaves.
 * @param wasm - Circuits wasm.
 * @param fnLeves - The function leaves to be included in the contract function tree.
 * @returns The function tree root.
 */
export async function computeFunctionTreeRoot(wasm: CircuitsWasm, fnLeves: Fr[]) {
  const inputVector = serializeBufferArrayToVector(fnLeves.map(fr => fr.toBuffer()));
  wasm.call('pedersen__init');
  const result = await wasmAsyncCall(wasm, 'abis__compute_function_tree_root', inputVector, 32);
  return Fr.fromBuffer(result);
}

/**
 * Computes a constructor hash.
 * @param wasm - Circuits wasm.
 * @param functionData - Constructor's function data.
 * @param args - Constructor's arguments.
 * @param constructorVKHash - Hash of the constructor's verification key.
 * @returns The constructor hash.
 */
export async function hashConstructor(
  wasm: CircuitsWasm,
  functionData: FunctionData,
  args: Fr[],
  constructorVKHash: Buffer,
): Promise<Buffer> {
  if (args.length > ARGS_LENGTH) {
    throw new Error(`Expected constructor args to have length <= ${ARGS_LENGTH}! Was: ${args.length}`);
  }
  const numEmptyArgs = ARGS_LENGTH - args.length;
  const emptyArgs = Array.from({ length: numEmptyArgs }, () => new Fr(0n));
  const fullArgs = args.concat(emptyArgs);
  const inputVector = serializeToBuffer(fullArgs.map(fr => fr.toBuffer()));
  wasm.call('pedersen__init');
  const result = await inputBuffersToOutputBuffer(
    wasm,
    'abis__hash_constructor',
    [functionData.toBuffer(), inputVector, constructorVKHash],
    32,
  );
  return result;
}

/**
 * Computes a contract address.
 * @param wasm - Circuits wasm.
 * @param deployerAddr - The address of the contract deployer.
 * @param contractAddrSalt - The salt used as 1 one of the inputs of the contract address computation.
 * @param fnTreeRoot - The function tree root of the contract being deployed.
 * @param constructorHash - The hash of the constructor.
 * @returns The contract address.
 */
export async function computeContractAddress(
  wasm: CircuitsWasm,
  deployerAddr: AztecAddress,
  contractAddrSalt: Fr,
  fnTreeRoot: Fr,
  constructorHash: Buffer,
): Promise<AztecAddress> {
  const deployerAddrBuf = deployerAddr.toBuffer();
  wasm.call('pedersen__init');
  const result = await inputBuffersToOutputBuffer(
    wasm,
    'abis__compute_contract_address',
    [deployerAddrBuf, contractAddrSalt.toBuffer(), fnTreeRoot.toBuffer(), constructorHash],
    32,
  );
  return AztecAddress.fromBuffer(result);
}

/**
 * Computes a contract leaf of the given contract.
 * @param wasm - Relevant WASM wrapper.
 * @param cd - The contract data of the deployed contract.
 * @returns The contract leaf.
 */
export function computeContractLeaf(wasm: WasmWrapper, cd: NewContractData): Fr {
  wasm.call('pedersen__init');
  const value = wasmSyncCall(wasm, 'abis__compute_contract_leaf', cd, 32);
  return Fr.fromBuffer(value);
}

/**
 * Computes tx hash of a given transaction request.
 * @param wasm - Relevant WASM wrapper.
 * @param txRequest - The signed transaction request.
 * @returns The transaction hash.
 */
export function computeTxHash(wasm: WasmWrapper, txRequest: SignedTxRequest): Fr {
  wasm.call('pedersen__init');
  const value = wasmSyncCall(wasm, 'abis__compute_transaction_hash', txRequest, 32);
  return Fr.fromBuffer(value);
}

/**
 * Computes a call stack item hash.
 * @param wasm - Relevant WASM wrapper.
 * @param callStackItem - The call stack item.
 * @returns The call stack item hash.
 */
export function computeCallStackItemHash(wasm: WasmWrapper, callStackItem: PublicCallStackItem): Fr {
  wasm.call('pedersen__init');
  const value = wasmSyncCall(wasm, 'abis__compute_call_stack_item_hash', callStackItem, 32);
  return Fr.fromBuffer(value);
}
