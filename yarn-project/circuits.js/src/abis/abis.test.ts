import { randomBytes } from 'crypto';
import { fr, makeAztecAddress, makeTxRequest, makeVerificationKey } from '../tests/factories.js';
import { CircuitsWasm } from '../wasm/circuits_wasm.js';
import {
  computeContractAddress,
  computeContractLeaf,
  computeFunctionLeaf,
  computeFunctionSelector,
  computeFunctionTreeRoot,
  hashTxRequest,
  hashVK,
} from './abis.js';
import { NullifierLeafPreimage } from '../index.js';

describe('abis wasm bindings', () => {
  let wasm: CircuitsWasm;
  beforeEach(async () => {
    wasm = await CircuitsWasm.new();
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
    const leaf = Buffer.alloc(32);
    const res = computeFunctionLeaf(wasm, leaf);
    expect(res).toMatchSnapshot();
  });

  it('computes function tree root', () => {
    const res = computeFunctionTreeRoot(wasm, [Buffer.alloc(32), Buffer.alloc(32), Buffer.alloc(32), Buffer.alloc(32)]);
    expect(res).toMatchSnapshot();
  });

  it('computes a contract address', () => {
    const deployerAddr = makeAztecAddress(1);
    const contractAddrSalt = randomBytes(32);
    const treeRoot = randomBytes(32);
    const constructorHash = randomBytes(32);
    const res = computeContractAddress(wasm, deployerAddr, contractAddrSalt, treeRoot, constructorHash);
    expect(res).toMatchSnapshot();
  });

  it('computes contract leaf', () => {
    const leafPreImage = new NullifierLeafPreimage(fr(2), fr(2 + 0x100), 2 + 0x200);
    const res = computeContractLeaf(wasm, leafPreImage);
    expect(res).toMatchSnapshot();
  });
});
