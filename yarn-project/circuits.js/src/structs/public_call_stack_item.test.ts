import { randomInt } from '@aztec/foundation/crypto';
import { setupCustomSnapshotSerializers, updateInlineTestData } from '@aztec/foundation/testing';

import { makePublicCallStackItem } from '../tests/factories.js';
import { AztecAddress, Fr, FunctionData, FunctionSelector } from './index.js';
import { NoteHash } from './note_hash.js';
import { PublicCallStackItem } from './public_call_stack_item.js';

describe('PublicCallStackItem', () => {
  setupCustomSnapshotSerializers(expect);
  it('serializes to buffer and deserializes it back', () => {
    const expected = makePublicCallStackItem(randomInt(1000));
    const buffer = expected.toBuffer();
    const res = PublicCallStackItem.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });

  it('computes hash', () => {
    const seed = 9870243;
    const item = makePublicCallStackItem(seed);
    const hash = item.getCompressed().hash();
    expect(hash).toMatchSnapshot();
  });

  it('computes empty item hash', () => {
    const item = PublicCallStackItem.empty();
    const hash = item.getCompressed().hash();
    expect(hash).toMatchSnapshot();
  });

  it('Computes a callstack item request hash', () => {
    const callStack = PublicCallStackItem.empty();

    callStack.contractAddress = AztecAddress.fromField(new Fr(1));
    callStack.functionData = new FunctionData(new FunctionSelector(2), /*isPrivate=*/ false);
    callStack.isExecutionRequest = true;
    callStack.publicInputs.noteHashes[0] = new NoteHash(new Fr(1), 0);

    const hash = callStack.getCompressed().hash();
    expect(hash.toString()).toMatchSnapshot();

    // Run with AZTEC_GENERATE_TEST_DATA=1 to update noir test data
    updateInlineTestData(
      'noir-projects/noir-protocol-circuits/crates/types/src/abis/public_call_stack_item.nr',
      'test_data_call_stack_item_request_hash',
      hash.toString(),
    );
  });

  it('Computes a callstack item hash', () => {
    const callStack = PublicCallStackItem.empty();

    callStack.contractAddress = AztecAddress.fromField(new Fr(1));
    callStack.functionData = new FunctionData(new FunctionSelector(2), /*isPrivate=*/ false);
    callStack.publicInputs.noteHashes[0] = new NoteHash(new Fr(1), 0);

    const hash = callStack.getCompressed().hash();
    expect(hash.toString()).toMatchSnapshot();

    // Run with AZTEC_GENERATE_TEST_DATA=1 to update noir test data
    updateInlineTestData(
      'noir-projects/noir-protocol-circuits/crates/types/src/abis/public_call_stack_item.nr',
      'test_data_call_stack_item_hash',
      hash.toString(),
    );
  });
});
