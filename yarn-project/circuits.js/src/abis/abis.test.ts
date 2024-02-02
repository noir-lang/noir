import { times } from '@aztec/foundation/collection';

import {
  AztecAddress,
  Fr,
  FunctionData,
  FunctionLeafPreimage,
  FunctionSelector,
  NewContractData,
  PublicCallStackItem,
  SideEffect,
  SideEffectLinkedToNoteHash,
} from '../index.js';
import {
  makeAztecAddress,
  makeEthAddress,
  makePrivateCallStackItem,
  makePublicCallStackItem,
  makeTxRequest,
  makeVerificationKey,
} from '../tests/factories.js';
import {
  computeCommitmentNonce,
  computeCommitmentsHash,
  computeContractLeaf,
  computeFunctionLeaf,
  computeFunctionSelector,
  computeFunctionTreeRoot,
  computeNullifierHash,
  computePrivateCallStackItemHash,
  computePublicCallStackItemHash,
  computePublicDataTreeLeafSlot,
  computePublicDataTreeValue,
  computeSecretMessageHash,
  computeTxHash,
  computeUniqueCommitment,
  computeVarArgsHash,
  hashConstructor,
  hashTxRequest,
  hashVK,
  siloCommitment,
  siloNullifier,
} from './abis.js';

describe('abis', () => {
  it('hashes a tx request', () => {
    const txRequest = makeTxRequest();
    const hash = hashTxRequest(txRequest);
    expect(hash).toMatchSnapshot();
  });

  it('computes a function selector', () => {
    const funcSig = 'transfer(address,uint256)';
    const res = computeFunctionSelector(funcSig);
    expect(res).toMatchSnapshot();
  });

  it('hashes VK', () => {
    const vk = makeVerificationKey();
    const res = hashVK(vk.toBuffer());
    expect(res).toMatchSnapshot();
  });

  it('computes a function leaf', () => {
    const leaf = new FunctionLeafPreimage(new FunctionSelector(7837), false, true, Fr.ZERO, Fr.ZERO);
    const res = computeFunctionLeaf(leaf);
    expect(res).toMatchSnapshot();
  });

  it('computes function tree root', () => {
    const res = computeFunctionTreeRoot([new Fr(0n), new Fr(0n), new Fr(0n), new Fr(0n)]);
    expect(res).toMatchSnapshot();
  });

  it('hashes constructor info', () => {
    const functionData = new FunctionData(FunctionSelector.empty(), false, true, true);
    const argsHash = new Fr(42);
    const vkHash = Buffer.alloc(32);
    const res = hashConstructor(functionData, argsHash, vkHash);
    expect(res).toMatchSnapshot();
  });

  it('computes commitment nonce', () => {
    const nullifierZero = new Fr(123n);
    const commitmentIndex = 456;
    const res = computeCommitmentNonce(nullifierZero, commitmentIndex);
    expect(res).toMatchSnapshot();
  });

  it('computes unique commitment', () => {
    const nonce = new Fr(123n);
    const innerCommitment = new Fr(456);
    const res = computeUniqueCommitment(nonce, innerCommitment);
    expect(res).toMatchSnapshot();
  });

  it('computes siloed commitment', () => {
    const contractAddress = new AztecAddress(new Fr(123n).toBuffer());
    const uniqueCommitment = new Fr(456);
    const res = siloCommitment(contractAddress, uniqueCommitment);
    expect(res).toMatchSnapshot();
  });

  it('computes siloed nullifier', () => {
    const contractAddress = new AztecAddress(new Fr(123n).toBuffer());
    const innerNullifier = new Fr(456);
    const res = siloNullifier(contractAddress, innerNullifier);
    expect(res).toMatchSnapshot();
  });

  it('computes public data tree value', () => {
    const value = new Fr(3n);
    const res = computePublicDataTreeValue(value);
    expect(res).toMatchSnapshot();
  });

  it('computes public data tree leaf slot', () => {
    const contractAddress = makeAztecAddress();
    const value = new Fr(3n);
    const res = computePublicDataTreeLeafSlot(contractAddress, value);
    expect(res).toMatchSnapshot();
  });

  it('hashes empty function args', () => {
    const res = computeVarArgsHash([]);
    expect(res).toMatchSnapshot();
  });

  it('hashes function args', () => {
    // const args = Array.from({ length: 8 }).map((_, i) => new Fr(i));
    const args = times(8, i => new Fr(i));
    const res = computeVarArgsHash(args);
    expect(res).toMatchSnapshot();
  });

  it('hashes many function args', () => {
    const args = times(200, i => new Fr(i));
    const res = computeVarArgsHash(args);
    expect(res).toMatchSnapshot();
  });

  it('computes contract leaf', () => {
    const cd = new NewContractData(makeAztecAddress(), makeEthAddress(), new Fr(3n));
    const res = computeContractLeaf(cd);
    expect(res).toMatchSnapshot();
  });

  it('computes zero contract leaf', () => {
    const cd = new NewContractData(AztecAddress.ZERO, AztecAddress.ZERO, new Fr(0n));
    const res = computeContractLeaf(cd);
    expect(res).toMatchSnapshot();
  });

  it('compute tx hash', () => {
    const txRequest = makeTxRequest();
    const hash = computeTxHash(txRequest);
    expect(hash).toMatchSnapshot();
  });

  it('compute private call stack item hash', () => {
    const item = makePrivateCallStackItem();
    const hash = computePrivateCallStackItemHash(item);
    expect(hash).toMatchSnapshot();
  });

  it('compute public call stack item hash', () => {
    const item = makePublicCallStackItem();
    const hash = computePublicCallStackItemHash(item);
    expect(hash).toMatchSnapshot();
  });

  it('compute secret message hash', () => {
    const value = new Fr(8n);
    const hash = computeSecretMessageHash(value);
    expect(hash).toMatchSnapshot();
  });

  it('Computes an empty nullifier hash ', () => {
    const emptyNull = SideEffectLinkedToNoteHash.empty();

    const emptyHash = Fr.fromBuffer(computeNullifierHash(emptyNull)).toString();
    expect(emptyHash).toMatchSnapshot();
  });

  it('Computes an empty sideeffect hash ', () => {
    const emptySideEffect = SideEffect.empty();
    const emptyHash = Fr.fromBuffer(computeCommitmentsHash(emptySideEffect)).toString();
    expect(emptyHash).toMatchSnapshot();
  });

  it('Computes a callstack item request hash', () => {
    const callStack = PublicCallStackItem.empty();

    callStack.contractAddress = AztecAddress.fromField(new Fr(1));
    callStack.functionData = new FunctionData(new FunctionSelector(2), false, false, false);
    callStack.isExecutionRequest = true;
    callStack.publicInputs.newCommitments[0] = new SideEffect(new Fr(1), new Fr(0));

    const hash = callStack.hash();
    expect(hash.toString()).toMatchSnapshot();

    // Value used in compute_call_stack_item_hash test in noir circuits
    // console.log("hash", hash.toString());
  });

  it('Computes a callstack item hash', () => {
    const callStack = PublicCallStackItem.empty();

    callStack.contractAddress = AztecAddress.fromField(new Fr(1));
    callStack.functionData = new FunctionData(new FunctionSelector(2), false, false, false);
    callStack.publicInputs.newCommitments[0] = new SideEffect(new Fr(1), new Fr(0));

    const hash = callStack.hash();
    expect(hash.toString()).toMatchSnapshot();

    // Value used in compute_call_stack_item_request_hash test in noir circuits
    // console.log("hash", hash.toString());
  });
});
