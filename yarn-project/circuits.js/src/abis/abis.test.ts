import { ARGS_LENGTH, Fr, FunctionData, FunctionLeafPreimage, NewContractData } from '../index.js';
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
  beforeAll(async () => {
    wasm = await CircuitsWasm.get();
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
    const leaf = new FunctionLeafPreimage(Buffer.from([0, 0, 0, 123]), true, Fr.ZERO, Fr.ZERO);
    const res = await computeFunctionLeaf(wasm, leaf);
    expect(res).toMatchSnapshot();
  });

  it('compute function leaf should revert if buffer is over 4 bytes', () => {
    expect(() => {
      new FunctionLeafPreimage(Buffer.from([0, 0, 0, 0, 123]), true, Fr.ZERO, Fr.ZERO);
    }).toThrow('Function selector must be 4 bytes long, got 5 bytes.');
  });

  it('function leaf toBuffer should revert if buffer is over 4 bytes ', () => {
    const initBuffer = Buffer.from([0, 0, 0, 123]);
    const largerBuffer = Buffer.from([0, 0, 0, 0, 123]);
    expect(() => {
      const leaf = new FunctionLeafPreimage(initBuffer, true, Fr.ZERO, Fr.ZERO);
      leaf.functionSelector = largerBuffer;
      leaf.toBuffer();
    }).toThrow('Function selector must be 4 bytes long, got 5 bytes.');
  });

  it('computes function tree root', async () => {
    const res = await computeFunctionTreeRoot(wasm, [new Fr(0n), new Fr(0n), new Fr(0n), new Fr(0n)]);
    expect(res).toMatchSnapshot();
  });

  it('hash constructor info 2 args', async () => {
    const functionData = new FunctionData(Buffer.alloc(4), true, true);
    // args needs to have a FIXED length of 8, due to a circuit constant `aztec3::ARGS_SIZE`.
    const args = [new Fr(0n), new Fr(1n)];
    const vkHash = Buffer.alloc(32);
    const res = await hashConstructor(wasm, functionData, args, vkHash);
    expect(res).toMatchSnapshot();
  });

  it('hash constructor info (max args)', async () => {
    const functionData = new FunctionData(Buffer.alloc(4), true, true);
    const args = Array.from({ length: ARGS_LENGTH }, (v, i) => new Fr(BigInt(i)));
    const vkHash = Buffer.alloc(32);
    const res = await hashConstructor(wasm, functionData, args, vkHash);
    expect(res).toMatchSnapshot();
  });

  it('hash constructor throws (>max args)', async () => {
    const functionData = new FunctionData(Buffer.alloc(4), true, true);
    const args = Array.from({ length: ARGS_LENGTH + 1 }, (v, i) => new Fr(BigInt(i)));
    const vkHash = Buffer.alloc(32);
    await expect(hashConstructor(wasm, functionData, args, vkHash)).rejects.toThrow();
  });

  it('computes a contract address', async () => {
    const deployerAddr = makeAztecAddress(1);
    const contractAddrSalt = new Fr(2n);
    const treeRoot = new Fr(3n);
    const constructorHash = makeBytes();
    const res = await computeContractAddress(wasm, deployerAddr, contractAddrSalt, treeRoot, constructorHash);
    expect(res).toMatchSnapshot();
  });

  it('computes contract leaf', () => {
    const cd = new NewContractData(makeAztecAddress(), makeEthAddress(), new Fr(3n));
    const res = computeContractLeaf(wasm, cd);
    expect(res).toMatchSnapshot();
  });
});
