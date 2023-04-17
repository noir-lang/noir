import { Buffer } from 'buffer';
import { AztecAddress, Fr } from '@aztec/foundation';
import { CircuitsWasm } from '../wasm/index.js';
import { FunctionData, FUNCTION_SELECTOR_NUM_BYTES, TxRequest, NewContractData } from '../index.js';
import { serializeToBuffer } from '../utils/serialize.js';
import { AsyncWasmWrapper, WasmWrapper } from '@aztec/foundation/wasm';

export function wasmSyncCall(
  wasm: WasmWrapper,
  fnName: string,
  input: { toBuffer: () => Buffer },
  expectedOutputLength: number,
): Buffer {
  const inputData = input.toBuffer();
  const outputBuf = wasm.call('bbmalloc', expectedOutputLength);
  const inputBuf = wasm.call('bbmalloc', inputData.length);
  wasm.writeMemory(inputBuf, inputData);
  wasm.call(fnName, inputBuf, outputBuf);
  const buf = Buffer.from(wasm.getMemorySlice(outputBuf, outputBuf + expectedOutputLength));
  wasm.call('bbfree', outputBuf);
  wasm.call('bbfree', inputBuf);
  return buf;
}

export async function wasmAsyncCall(
  wasm: AsyncWasmWrapper,
  fnName: string,
  input: { toBuffer: () => Buffer },
  expectedOutputLength: number,
): Promise<Buffer> {
  const inputData = input.toBuffer();
  const outputBuf = wasm.call('bbmalloc', expectedOutputLength);
  const inputBuf = wasm.call('bbmalloc', inputData.length);
  wasm.writeMemory(inputBuf, inputData);
  await wasm.asyncCall(fnName, inputBuf, outputBuf);
  const buf = Buffer.from(wasm.getMemorySlice(outputBuf, outputBuf + expectedOutputLength));
  wasm.call('bbfree', outputBuf);
  wasm.call('bbfree', inputBuf);
  return buf;
}

export async function inputBuffersToOutputBuffer(
  wasm: CircuitsWasm,
  fnName: string,
  buffers: Buffer[],
  expectedOutputLength: number,
) {
  const offsets: number[] = [];
  const totalLength = buffers.reduce((total, cur) => {
    offsets.push(total);
    return total + cur.length;
  }, 0);

  const outputBuf = wasm.call('bbmalloc', expectedOutputLength);
  const inputBuf = wasm.call('bbmalloc', totalLength);
  wasm.writeMemory(inputBuf, Buffer.concat(buffers));
  const args = offsets.map(offset => inputBuf + offset);
  await wasm.asyncCall(fnName, ...args, outputBuf);
  const output = Buffer.from(wasm.getMemorySlice(outputBuf, outputBuf + expectedOutputLength));
  wasm.call('bbfree', inputBuf);
  wasm.call('bbfree', outputBuf);
  return output;
}

export async function hashTxRequest(wasm: CircuitsWasm, txRequest: TxRequest) {
  wasm.call('pedersen__init');
  return await wasmAsyncCall(wasm, 'abis__hash_tx_request', txRequest, 32);
}

export async function computeFunctionSelector(wasm: CircuitsWasm, funcSig: string) {
  return await wasmAsyncCall(
    wasm,
    'abis__compute_function_selector',
    { toBuffer: () => Buffer.from(funcSig) },
    FUNCTION_SELECTOR_NUM_BYTES,
  );
}

export async function hashVK(wasm: CircuitsWasm, vkBuf: Buffer) {
  wasm.call('pedersen__init');
  return await wasmAsyncCall(wasm, 'abis__hash_vk', { toBuffer: () => vkBuf }, 32);
}

export async function computeFunctionLeaf(wasm: CircuitsWasm, fnLeaf: Buffer) {
  // Size must match circuits/cpp/src/aztec3/circuits/abis/function_leaf_preimage.hpp
  if (fnLeaf.length !== 32 + 1 + 32 + 32) throw new Error(`Invalid length for function leaf`);
  wasm.call('pedersen__init');
  return Fr.fromBuffer(await wasmAsyncCall(wasm, 'abis__compute_function_leaf', { toBuffer: () => fnLeaf }, 32));
}

export async function computeFunctionTreeRoot(wasm: CircuitsWasm, fnLeafs: Fr[]) {
  const inputBuf = serializeToBuffer(fnLeafs);
  wasm.call('pedersen__init');
  const outputBuf = wasm.call('bbmalloc', 32);
  const inputBufPtr = wasm.call('bbmalloc', inputBuf.length);
  wasm.writeMemory(inputBufPtr, inputBuf);
  await wasm.asyncCall('abis__compute_function_tree_root', inputBufPtr, fnLeafs.length, outputBuf);
  return Fr.fromBuffer(Buffer.from(wasm.getMemorySlice(outputBuf, outputBuf + 32)));
}

export async function hashConstructor(
  wasm: CircuitsWasm,
  functionData: FunctionData,
  args: Fr[],
  constructorVKHash: Buffer,
) {
  const inputVector = serializeToBuffer(args.map(fr => fr.toBuffer()));
  wasm.call('pedersen__init');
  const result = await inputBuffersToOutputBuffer(
    wasm,
    'abis__hash_constructor',
    [functionData.toBuffer(), inputVector, constructorVKHash],
    32,
  );
  return result;
}

export async function computeContractAddress(
  wasm: CircuitsWasm,
  deployerAddr: AztecAddress,
  contractAddrSalt: Fr,
  fnTreeRoot: Fr,
  constructorHash: Buffer,
) {
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

export function computeContractLeaf(wasm: WasmWrapper, cd: NewContractData) {
  wasm.call('pedersen__init');
  const value = wasmSyncCall(wasm, 'abis__compute_contract_leaf', { toBuffer: () => cd.toBuffer() }, 32);
  return Fr.fromBuffer(value);
}
