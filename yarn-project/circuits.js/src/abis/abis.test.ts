import { Fr, FunctionData, NewContractData } from '../index.js';
import { makeEthAddress } from '../tests/factories.js';
import { makeAztecAddress, makeBytes, makeTxRequest, makeVerificationKey } from '../tests/factories.js';
import { CircuitsWasm } from '../wasm/circuits_wasm.js';
import {
  computeContractAddress,
  computeContractLeaf,
  computeFunctionLeaf,
  computeFunctionSelector,
  computeFunctionTreeRoot,
  hashConstructor,
  hashTxRequest,
  hashVK,
} from './abis.js';

describe('abis wasm bindings', () => {
  let wasm: CircuitsWasm;
  beforeEach(async () => {
    wasm = await CircuitsWasm.new();
  });

  it('hashes a tx request', async () => {
    const txRequest = makeTxRequest();
    const hash = await hashTxRequest(wasm, txRequest);
    expect(hash).toMatchSnapshot();
  });

  it('computes a function selector', async () => {
    const funcSig = 'transfer(address,uint256)';
    const res = await computeFunctionSelector(wasm, funcSig);
    expect(res).toMatchSnapshot();
  });

  // TODO: This test fails on CI since build-system is not updating the latest circuits wasm
  // We may need to wait until we bump to the next commit to see if it picks up the change
  it.skip('hashes VK', async () => {
    const vk = makeVerificationKey();
    const res = await hashVK(wasm, vk.toBuffer());
    expect(res).toMatchSnapshot();
  });

  it('computes a function leaf', async () => {
    const leaf = Buffer.alloc(32);
    const res = await computeFunctionLeaf(wasm, leaf);
    expect(res).toMatchSnapshot();
  });

  it('computes function tree root', async () => {
    const res = await computeFunctionTreeRoot(wasm, [
      Buffer.alloc(32),
      Buffer.alloc(32),
      Buffer.alloc(32),
      Buffer.alloc(32),
    ]);
    expect(res).toMatchSnapshot();
  });

  it('hash constructor info', async () => {
    const functionData = new FunctionData(Buffer.alloc(4), true, true);
    const args = [new Fr(0n), new Fr(1n)];
    const vkHash = Buffer.alloc(32);
    const res = await hashConstructor(wasm, functionData, args, vkHash);
    expect(res).toMatchSnapshot();
  });

  it('computes a contract address', async () => {
    const deployerAddr = makeAztecAddress(1);
    const contractAddrSalt = makeBytes();
    const treeRoot = makeBytes();
    const constructorHash = makeBytes();
    const res = await computeContractAddress(wasm, deployerAddr, contractAddrSalt, treeRoot, constructorHash);
    expect(res).toMatchSnapshot();
  });

  it('computes contract leaf', async () => {
    const cd = new NewContractData(makeAztecAddress(), makeEthAddress(), new Fr(3n));
    const res = await computeContractLeaf(wasm, cd);
    expect(res).toMatchSnapshot();
  });
});
