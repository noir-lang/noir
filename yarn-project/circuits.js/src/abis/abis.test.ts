import times from 'lodash.times';

import { AztecAddress, Fr, FunctionData, FunctionLeafPreimage, FunctionSelector, NewContractData } from '../index.js';
import { makeAztecAddress, makeEthAddress, makePoint, makeTxRequest, makeVerificationKey } from '../tests/factories.js';
import { CircuitsWasm } from '../wasm/circuits_wasm.js';
import {
  computeCommitmentNonce,
  computeContractAddress,
  computeContractLeaf,
  computeFunctionLeaf,
  computeFunctionSelector,
  computeFunctionTreeRoot,
  computeUniqueCommitment,
  computeVarArgsHash,
  hashConstructor,
  hashTxRequest,
  hashVK,
  siloCommitment,
  siloNullifier,
} from './abis.js';

describe('abis wasm bindings', () => {
  let wasm: CircuitsWasm;
  beforeAll(async () => {
    wasm = await CircuitsWasm.get();
  });

  it('hashes a tx request', () => {
    const txRequest = makeTxRequest();
    const hash = hashTxRequest(wasm, txRequest);
    expect(hash).toMatchSnapshot();
  });

  it('computes a function selector', () => {
    const funcSig = 'transfer(address,uint256)';
    const res = computeFunctionSelector(wasm, funcSig);
    expect(res).toMatchSnapshot();
  });

  it('hashes VK', () => {
    const vk = makeVerificationKey();
    const res = hashVK(wasm, vk.toBuffer());
    expect(res).toMatchSnapshot();
  });

  it('computes a function leaf', () => {
    const leaf = new FunctionLeafPreimage(new FunctionSelector(7837), false, true, Fr.ZERO, Fr.ZERO);
    const res = computeFunctionLeaf(wasm, leaf);
    expect(res).toMatchSnapshot();
  });

  it('computes function tree root', () => {
    const res = computeFunctionTreeRoot(wasm, [new Fr(0n), new Fr(0n), new Fr(0n), new Fr(0n)]);
    expect(res).toMatchSnapshot();
  });

  it('hashes constructor info', () => {
    const functionData = new FunctionData(FunctionSelector.empty(), false, true, true);
    const argsHash = new Fr(42);
    const vkHash = Buffer.alloc(32);
    const res = hashConstructor(wasm, functionData, argsHash, vkHash);
    expect(res).toMatchSnapshot();
  });

  it('computes a contract address', () => {
    const deployerPubKey = makePoint();
    const contractAddrSalt = new Fr(2n);
    const treeRoot = new Fr(3n);
    const constructorHash = new Fr(4n);
    const res = computeContractAddress(wasm, deployerPubKey, contractAddrSalt, treeRoot, constructorHash);
    expect(res).toMatchSnapshot();
  });

  it('computes commitment nonce', () => {
    const nullifierZero = new Fr(123n);
    const commitmentIndex = 456;
    const res = computeCommitmentNonce(wasm, nullifierZero, commitmentIndex);
    expect(res).toMatchSnapshot();
  });

  it('computes unique commitment', () => {
    const nonce = new Fr(123n);
    const innerCommitment = new Fr(456);
    const res = computeUniqueCommitment(wasm, nonce, innerCommitment);
    expect(res).toMatchSnapshot();
  });

  it('computes siloed commitment', () => {
    const contractAddress = new AztecAddress(new Fr(123n).toBuffer());
    const uniqueCommitment = new Fr(456);
    const res = siloCommitment(wasm, contractAddress, uniqueCommitment);
    expect(res).toMatchSnapshot();
  });

  it('computes siloed nullifier', () => {
    const contractAddress = new AztecAddress(new Fr(123n).toBuffer());
    const innerNullifier = new Fr(456);
    const res = siloNullifier(wasm, contractAddress, innerNullifier);
    expect(res).toMatchSnapshot();
  });

  it('computes contract leaf', () => {
    const cd = new NewContractData(makeAztecAddress(), makeEthAddress(), new Fr(3n));
    const res = computeContractLeaf(wasm, cd);
    expect(res).toMatchSnapshot();
  });

  it('hashes empty function args', async () => {
    const res = await computeVarArgsHash(wasm, []);
    expect(res).toMatchSnapshot();
  });

  it('hashes function args', async () => {
    const args = times(8, i => new Fr(i));
    const res = await computeVarArgsHash(wasm, args);
    expect(res).toMatchSnapshot();
  });

  it('hashes many function args', async () => {
    const args = times(200, i => new Fr(i));
    const res = await computeVarArgsHash(wasm, args);
    expect(res).toMatchSnapshot();
  });
});
