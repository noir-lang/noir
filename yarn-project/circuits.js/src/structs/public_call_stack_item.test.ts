import { makePublicCallStackItem } from '../tests/factories.js';
import { AztecAddress, Fr, FunctionData, FunctionSelector, SideEffect } from './index.js';
import { PublicCallStackItem } from './public_call_stack_item.js';

describe('PublicCallStackItem', () => {
  it('serializes to buffer and deserializes it back', () => {
    const randomInt = Math.floor(Math.random() * 1000);
    const expected = makePublicCallStackItem(randomInt);
    const buffer = expected.toBuffer();
    const res = PublicCallStackItem.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });

  it('computes hash', () => {
    const seed = 9870243;
    const item = makePublicCallStackItem(seed);
    const hash = item.hash();
    expect(hash).toMatchSnapshot();
  });

  it('Computes a callstack item request hash', () => {
    const callStack = PublicCallStackItem.empty();

    callStack.contractAddress = AztecAddress.fromField(new Fr(1));
    callStack.functionData = new FunctionData(new FunctionSelector(2), false, false, false);
    callStack.isExecutionRequest = true;
    callStack.publicInputs.newNoteHashes[0] = new SideEffect(new Fr(1), new Fr(0));

    const hash = callStack.hash();
    expect(hash.toString()).toMatchSnapshot();

    // Value used in compute_call_stack_item_hash test in public_call_stack_item.test.ts
    // console.log("hash", hash.toString());
  });

  it('Computes a callstack item hash', () => {
    const callStack = PublicCallStackItem.empty();

    callStack.contractAddress = AztecAddress.fromField(new Fr(1));
    callStack.functionData = new FunctionData(new FunctionSelector(2), false, false, false);
    callStack.publicInputs.newNoteHashes[0] = new SideEffect(new Fr(1), new Fr(0));

    const hash = callStack.hash();
    expect(hash.toString()).toMatchSnapshot();

    // Value used in compute_call_stack_item_request_hash test in public_call_stack_item.test.ts
    // console.log("hash", hash.toString());
  });
});
