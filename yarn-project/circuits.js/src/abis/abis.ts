import { Buffer } from 'buffer';
import { AztecAddress, Fr, serializeBufferArrayToVector } from '@aztec/foundation';
import { CircuitsWasm } from '../wasm/index.js';
import { FunctionData, FUNCTION_SELECTOR_NUM_BYTES, NullifierLeafPreimage, TxRequest } from '../index.js';
import { serializeToBuffer } from '../utils/serialize.js';

export async function hashTxRequest(wasm: CircuitsWasm, txRequest: TxRequest) {
  const data = txRequest.toBuffer();
  wasm.call('pedersen__init');
  wasm.writeMemory(0, data);
  await wasm.asyncCall('abis__hash_tx_request', 0, data.length);
  return Buffer.from(wasm.getMemorySlice(data.length, data.length + 32));
}

export async function computeFunctionSelector(wasm: CircuitsWasm, funcSig: string) {
  const buf = Buffer.from(funcSig);
  wasm.writeMemory(0, buf);
  await wasm.asyncCall('abis__compute_function_selector', 0, buf.length);
  return Buffer.from(wasm.getMemorySlice(buf.length, buf.length + FUNCTION_SELECTOR_NUM_BYTES));
}

export async function hashVK(wasm: CircuitsWasm, vkBuf: Buffer) {
  wasm.call('pedersen__init');
  wasm.writeMemory(0, vkBuf);
  await wasm.asyncCall('abis__hash_vk', 0, vkBuf.length);
  return Buffer.from(wasm.getMemorySlice(vkBuf.length, vkBuf.length + 32));
}

export async function computeFunctionLeaf(wasm: CircuitsWasm, fnLeaf: Buffer) {
  wasm.call('pedersen__init');
  wasm.writeMemory(0, fnLeaf);
  await wasm.asyncCall('abis__compute_function_leaf', 0, fnLeaf.length);
  return Buffer.from(wasm.getMemorySlice(fnLeaf.length, fnLeaf.length + 32));
}

export async function computeFunctionTreeRoot(wasm: CircuitsWasm, fnLeafs: Buffer[]) {
  const inputVector = serializeBufferArrayToVector(fnLeafs);
  wasm.call('pedersen__init');
  wasm.writeMemory(0, inputVector);
  await wasm.asyncCall('abis__compute_function_tree_root', 0, fnLeafs.length, inputVector.length);
  return Buffer.from(wasm.getMemorySlice(inputVector.length, inputVector.length + 32));
}

export async function hashConstructor(
  wasm: CircuitsWasm,
  functionData: FunctionData,
  args: Fr[],
  constructorVKHash: Buffer,
) {
  const functionDataBuf = functionData.toBuffer();
  // writes length to buffer output
  const inputVector = serializeToBuffer(args.map(fr => fr.toBuffer()));
  const memLoc1 = functionDataBuf.length;
  const memLoc2 = memLoc1 + inputVector.length;
  const memLoc3 = memLoc2 + constructorVKHash.length;
  wasm.call('pedersen__init');
  wasm.writeMemory(0, functionDataBuf);
  wasm.writeMemory(memLoc1, inputVector);
  wasm.writeMemory(memLoc2, constructorVKHash);
  await wasm.asyncCall('abis__hash_constructor', 0, memLoc1, memLoc2, memLoc3);
  return Buffer.from(wasm.getMemorySlice(memLoc3, memLoc3 + 32));
}

export async function computeContractAddress(
  wasm: CircuitsWasm,
  deployerAddr: AztecAddress,
  contractAddrSalt: Buffer,
  fnTreeRoot: Buffer,
  constructorHash: Buffer,
) {
  const deployerAddrBuf = deployerAddr.toBuffer();
  const memLoc1 = deployerAddrBuf.length;
  const memLoc2 = memLoc1 + contractAddrSalt.length;
  const memLoc3 = memLoc2 + fnTreeRoot.length;
  const memLoc4 = memLoc3 + constructorHash.length;
  wasm.call('pedersen__init');
  wasm.writeMemory(0, deployerAddrBuf);
  wasm.writeMemory(memLoc1, contractAddrSalt);
  wasm.writeMemory(memLoc2, fnTreeRoot);
  wasm.writeMemory(memLoc3, constructorHash);
  await wasm.asyncCall('abis__compute_contract_address', 0, memLoc1, memLoc2, memLoc3, memLoc4);
  const resultBuf = Buffer.from(wasm.getMemorySlice(memLoc4, memLoc4 + 32));
  return AztecAddress.fromBuffer(resultBuf);
}

export async function computeContractLeaf(wasm: CircuitsWasm, leafPreimage: NullifierLeafPreimage) {
  const data = leafPreimage.toBuffer();
  wasm.call('pedersen__init');
  wasm.writeMemory(0, leafPreimage.toBuffer());
  await wasm.asyncCall('abis__compute_contract_leaf', 0, data.length);
  return Buffer.from(wasm.getMemorySlice(data.length, data.length + 32));
}
