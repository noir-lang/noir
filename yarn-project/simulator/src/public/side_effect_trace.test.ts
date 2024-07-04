import { UnencryptedL2Log } from '@aztec/circuit-types';
import { AztecAddress, EthAddress, Gas, L2ToL1Message } from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation/fields';
import { SerializableContractInstance } from '@aztec/types/contracts';

import { randomBytes, randomInt } from 'crypto';

import { AvmContractCallResult } from '../avm/avm_contract_call_result.js';
import { initExecutionEnvironment } from '../avm/fixtures/index.js';
import { PublicSideEffectTrace, type TracedContractInstance } from './side_effect_trace.js';

function randomTracedContractInstance(): TracedContractInstance {
  const instance = SerializableContractInstance.random();
  const address = AztecAddress.random();
  return { exists: true, ...instance, address };
}

describe('Side Effect Trace', () => {
  const address = Fr.random();
  const utxo = Fr.random();
  const leafIndex = Fr.random();
  const slot = Fr.random();
  const value = Fr.random();
  const recipient = Fr.random();
  const content = Fr.random();
  const log = [Fr.random(), Fr.random(), Fr.random()];

  const startGasLeft = Gas.fromFields([new Fr(randomInt(10000)), new Fr(randomInt(10000))]);
  const endGasLeft = Gas.fromFields([new Fr(randomInt(10000)), new Fr(randomInt(10000))]);
  const transactionFee = Fr.random();
  const calldata = [Fr.random(), Fr.random(), Fr.random(), Fr.random()];
  const bytecode = randomBytes(100);
  const returnValues = [Fr.random(), Fr.random()];

  const avmEnvironment = initExecutionEnvironment({
    address,
    calldata,
    transactionFee,
  });
  const reverted = false;
  const avmCallResults = new AvmContractCallResult(reverted, returnValues);

  let startCounter: number;
  let startCounterFr: Fr;
  let startCounterPlus1: number;
  let trace: PublicSideEffectTrace;

  beforeEach(() => {
    startCounter = randomInt(/*max=*/ 1000000);
    startCounterFr = new Fr(startCounter);
    startCounterPlus1 = startCounter + 1;
    trace = new PublicSideEffectTrace(startCounter);
  });

  const toPxResult = (trc: PublicSideEffectTrace) => {
    return trc.toPublicExecutionResult(avmEnvironment, startGasLeft, endGasLeft, bytecode, avmCallResults);
  };

  it('Should trace storage reads', () => {
    const exists = true;
    const cached = false;
    trace.tracePublicStorageRead(address, slot, value, exists, cached);
    expect(trace.getCounter()).toBe(startCounterPlus1);

    const pxResult = toPxResult(trace);
    expect(pxResult.contractStorageReads).toEqual([
      {
        storageSlot: slot,
        currentValue: value,
        counter: startCounter,
        contractAddress: AztecAddress.fromField(address),
        //exists: exists,
        //cached: cached,
      },
    ]);
    expect(pxResult.avmCircuitHints.storageValues.items).toEqual([{ key: startCounterFr, value: value }]);
  });

  it('Should trace storage writes', () => {
    trace.tracePublicStorageWrite(address, slot, value);
    expect(trace.getCounter()).toBe(startCounterPlus1);

    const pxResult = toPxResult(trace);
    expect(pxResult.contractStorageUpdateRequests).toEqual([
      {
        storageSlot: slot,
        newValue: value,
        counter: startCounter,
        contractAddress: AztecAddress.fromField(address),
      },
    ]);
  });

  it('Should trace note hash checks', () => {
    const exists = true;
    trace.traceNoteHashCheck(address, utxo, leafIndex, exists);
    expect(trace.getCounter()).toBe(startCounterPlus1);

    const pxResult = toPxResult(trace);
    expect(pxResult.noteHashReadRequests).toEqual([
      {
        //storageAddress: contractAddress,
        value: utxo,
        //exists: exists,
        counter: startCounter,
        //leafIndex: leafIndex,
      },
    ]);
    expect(pxResult.avmCircuitHints.noteHashExists.items).toEqual([{ key: startCounterFr, value: new Fr(exists) }]);
  });

  it('Should trace note hashes', () => {
    trace.traceNewNoteHash(address, utxo);
    expect(trace.getCounter()).toBe(startCounterPlus1);

    const pxResult = toPxResult(trace);
    expect(pxResult.noteHashes).toEqual([
      {
        //storageAddress: contractAddress,
        value: utxo,
        counter: startCounter,
      },
    ]);
  });

  it('Should trace nullifier checks', () => {
    const exists = true;
    const isPending = false;
    trace.traceNullifierCheck(address, utxo, leafIndex, exists, isPending);
    expect(trace.getCounter()).toBe(startCounterPlus1);

    const pxResult = toPxResult(trace);
    expect(pxResult.nullifierReadRequests).toEqual([
      {
        value: utxo,
        counter: startCounter,
      },
    ]);
    expect(pxResult.nullifierNonExistentReadRequests).toEqual([]);
    expect(pxResult.avmCircuitHints.nullifierExists.items).toEqual([{ key: startCounterFr, value: new Fr(exists) }]);
  });

  it('Should trace non-existent nullifier checks', () => {
    const exists = false;
    const isPending = false;
    trace.traceNullifierCheck(address, utxo, leafIndex, exists, isPending);
    expect(trace.getCounter()).toBe(startCounterPlus1);

    const pxResult = toPxResult(trace);
    expect(pxResult.nullifierReadRequests).toEqual([]);
    expect(pxResult.nullifierNonExistentReadRequests).toEqual([
      {
        value: utxo,
        counter: startCounter,
      },
    ]);
    expect(pxResult.avmCircuitHints.nullifierExists.items).toEqual([{ key: startCounterFr, value: new Fr(exists) }]);
  });

  it('Should trace nullifiers', () => {
    trace.traceNewNullifier(address, utxo);
    expect(trace.getCounter()).toBe(startCounterPlus1);

    const pxResult = toPxResult(trace);
    expect(pxResult.nullifiers).toEqual([
      {
        value: utxo,
        counter: startCounter,
        noteHash: Fr.ZERO,
      },
    ]);
  });

  it('Should trace L1ToL2 Message checks', () => {
    const exists = true;
    trace.traceL1ToL2MessageCheck(address, utxo, leafIndex, exists);
    expect(trace.getCounter()).toBe(startCounterPlus1);

    const pxResult = toPxResult(trace);
    expect(pxResult.l1ToL2MsgReadRequests).toEqual([
      {
        value: utxo,
        counter: startCounter,
      },
    ]);
    expect(pxResult.avmCircuitHints.l1ToL2MessageExists.items).toEqual([
      {
        key: startCounterFr,
        value: new Fr(exists),
      },
    ]);
  });

  it('Should trace new L2ToL1 messages', () => {
    trace.traceNewL2ToL1Message(recipient, content);
    expect(trace.getCounter()).toBe(startCounterPlus1);

    const pxResult = toPxResult(trace);
    expect(pxResult.l2ToL1Messages).toEqual([
      new L2ToL1Message(EthAddress.fromField(recipient), content, startCounter),
    ]);
  });

  it('Should trace new unencrypted logs', () => {
    trace.traceUnencryptedLog(address, log);
    expect(trace.getCounter()).toBe(startCounterPlus1);

    const pxResult = toPxResult(trace);
    const expectLog = new UnencryptedL2Log(AztecAddress.fromField(address), Buffer.concat(log.map(f => f.toBuffer())));
    expect(pxResult.unencryptedLogs.logs).toEqual([expectLog]);
    expect(pxResult.allUnencryptedLogs.logs).toEqual([expectLog]);
    expect(pxResult.unencryptedLogsHashes).toEqual([
      expect.objectContaining({
        counter: startCounter,
      }),
    ]);
  });

  it('Should trace get contract instance', () => {
    const instance = randomTracedContractInstance();
    const { version: _, ...instanceWithoutVersion } = instance;
    trace.traceGetContractInstance(instance);
    expect(trace.getCounter()).toBe(startCounterPlus1);

    const pxResult = toPxResult(trace);
    // TODO(dbanks12): process contract instance read requests in public kernel
    //expect(pxResult.gotContractInstances).toEqual([instance]);
    expect(pxResult.avmCircuitHints.contractInstances.items).toEqual([
      {
        // hint omits "version" and has "exists" as an Fr
        ...instanceWithoutVersion,
        exists: new Fr(instance.exists),
      },
    ]);
  });

  it('Should trace nested calls', () => {
    const existsDefault = true;
    const cached = false;
    const isPending = false;

    const nestedTrace = new PublicSideEffectTrace(startCounter);
    let testCounter = startCounter;
    nestedTrace.tracePublicStorageRead(address, slot, value, existsDefault, cached);
    testCounter++;
    nestedTrace.tracePublicStorageWrite(address, slot, value);
    testCounter++;
    nestedTrace.traceNoteHashCheck(address, utxo, leafIndex, existsDefault);
    testCounter++;
    nestedTrace.traceNewNoteHash(address, utxo);
    testCounter++;
    nestedTrace.traceNullifierCheck(address, utxo, leafIndex, /*exists=*/ true, isPending);
    testCounter++;
    nestedTrace.traceNullifierCheck(address, utxo, leafIndex, /*exists=*/ false, isPending);
    testCounter++;
    nestedTrace.traceNewNullifier(address, utxo);
    testCounter++;
    nestedTrace.traceL1ToL2MessageCheck(address, utxo, leafIndex, existsDefault);
    testCounter++;
    nestedTrace.traceNewL2ToL1Message(recipient, content);
    testCounter++;
    nestedTrace.traceUnencryptedLog(address, log);
    testCounter++;

    trace.traceNestedCall(nestedTrace, avmEnvironment, startGasLeft, endGasLeft, bytecode, avmCallResults);
    // parent trace adopts nested call's counter
    expect(trace.getCounter()).toBe(testCounter);

    // get parent trace as result
    const parentPxResult = toPxResult(trace);
    const childPxResult = toPxResult(nestedTrace);
    expect(parentPxResult.nestedExecutions).toEqual([childPxResult]);

    // parent absorb's child's unencryptedLogs into all*
    expect(parentPxResult.allUnencryptedLogs).toEqual(childPxResult.allUnencryptedLogs);
  });
});
