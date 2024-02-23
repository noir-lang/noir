import { Fr } from '@aztec/foundation/fields';

import { WorldStateAccessTrace } from './trace.js';

describe('world state access trace', () => {
  let trace: WorldStateAccessTrace;

  beforeEach(() => {
    trace = new WorldStateAccessTrace();
  });

  describe('UTXOs', () => {
    it('Should trace commitments', () => {
      const contractAddress = new Fr(1);
      const utxo = new Fr(2);
      trace.traceNewNoteHash(contractAddress, utxo);
      expect(trace.newNoteHashes).toEqual([utxo]);
      expect(trace.getAccessCounter()).toEqual(1);
    });

    it('Should trace nullifiers', () => {
      const contractAddress = new Fr(1);
      const utxo = new Fr(2);
      trace.traceNewNullifier(contractAddress, utxo);
      expect(trace.newNullifiers).toEqual([utxo]);
      expect(trace.getAccessCounter()).toEqual(1);
    });
  });

  it('Should merge two traces together', () => {
    const contractAddress = new Fr(1);
    const slot = new Fr(2);
    const value = new Fr(1);
    const valueT1 = new Fr(2);
    const commitment = new Fr(10);
    const commitmentT1 = new Fr(20);

    trace.tracePublicStorageWrite(contractAddress, slot, value);
    trace.tracePublicStorageRead(contractAddress, slot, value);
    trace.traceNewNoteHash(contractAddress, commitment);
    trace.traceNewNullifier(contractAddress, commitment);
    expect(trace.getAccessCounter()).toEqual(4);

    const childTrace = new WorldStateAccessTrace(trace);
    childTrace.tracePublicStorageWrite(contractAddress, slot, valueT1);
    childTrace.tracePublicStorageRead(contractAddress, slot, valueT1);
    childTrace.traceNewNoteHash(contractAddress, commitmentT1);
    childTrace.traceNewNullifier(contractAddress, commitmentT1);
    expect(childTrace.getAccessCounter()).toEqual(8);

    trace.acceptAndMerge(childTrace);
    expect(trace.getAccessCounter()).toEqual(8);

    const slotReads = trace.publicStorageReads?.get(contractAddress.toBigInt())?.get(slot.toBigInt());
    const slotWrites = trace.publicStorageWrites?.get(contractAddress.toBigInt())?.get(slot.toBigInt());
    expect(slotReads).toEqual([value, valueT1]);
    expect(slotWrites).toEqual([value, valueT1]);
    expect(trace.newNoteHashes).toEqual([commitment, commitmentT1]);
    expect(trace.newNullifiers).toEqual([commitment, commitmentT1]);
  });
});
