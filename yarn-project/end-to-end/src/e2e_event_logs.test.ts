import { type AccountWalletWithSecretKey, type AztecNode, Fr, L1EventPayload, TaggedLog } from '@aztec/aztec.js';
import { deriveMasterIncomingViewingSecretKey } from '@aztec/circuits.js';
import { makeTuple } from '@aztec/foundation/array';
import { TestLogContract } from '@aztec/noir-contracts.js';

import { jest } from '@jest/globals';

import { publicDeployAccounts, setup } from './fixtures/utils.js';

const TIMEOUT = 120_000;

describe('Logs', () => {
  let testLogContract: TestLogContract;
  jest.setTimeout(TIMEOUT);

  let wallets: AccountWalletWithSecretKey[];
  let node: AztecNode;

  let teardown: () => Promise<void>;

  beforeAll(async () => {
    ({ teardown, wallets, aztecNode: node } = await setup(2));

    await publicDeployAccounts(wallets[0], wallets.slice(0, 2));

    testLogContract = await TestLogContract.deploy(wallets[0]).send().deployed();
  });

  afterAll(() => teardown());

  describe('functionality around emitting an encrypted log', () => {
    it('emits a generic encrypted log and checks for correctness', async () => {
      const randomness = Fr.random();
      const eventTypeId = Fr.random();
      const preimage = makeTuple(6, Fr.random);

      const tx = await testLogContract.methods.emit_encrypted_log(randomness, eventTypeId, preimage).send().wait();

      const txEffect = await node.getTxEffect(tx.txHash);

      const encryptedLogs = txEffect!.encryptedLogs.unrollLogs();
      expect(encryptedLogs.length).toBe(1);

      const decryptedLog = TaggedLog.decryptAsIncoming(
        encryptedLogs[0],
        deriveMasterIncomingViewingSecretKey(wallets[0].getSecretKey()),
        L1EventPayload,
      );

      expect(decryptedLog?.payload.contractAddress).toStrictEqual(testLogContract.address);
      expect(decryptedLog?.payload.randomness).toStrictEqual(randomness);
      expect(decryptedLog?.payload.eventTypeId).toStrictEqual(eventTypeId);
      expect(decryptedLog?.payload.event.items).toStrictEqual(preimage);
    });

    it('emits multiple events as encrypted logs and decodes them', async () => {
      const randomness = makeTuple(2, Fr.random);
      const preimage = makeTuple(4, Fr.random);

      const tx = await testLogContract.methods.emit_encrypted_events(randomness, preimage).send().wait();

      const txEffect = await node.getTxEffect(tx.txHash);

      const encryptedLogs = txEffect!.encryptedLogs.unrollLogs();
      expect(encryptedLogs.length).toBe(2);

      const decryptedLog0 = TaggedLog.decryptAsIncoming(
        encryptedLogs[0],
        deriveMasterIncomingViewingSecretKey(wallets[0].getSecretKey()),
        L1EventPayload,
      );

      expect(decryptedLog0?.payload.contractAddress).toStrictEqual(testLogContract.address);
      expect(decryptedLog0?.payload.randomness).toStrictEqual(randomness[0]);
      expect(decryptedLog0?.payload.eventTypeId).toStrictEqual(
        new Fr(0x00000000000000000000000000000000000000000000000000000000aa533f60),
      );

      // We decode our event into the event type
      const event0 = TestLogContract.events.ExampleEvent0.decode(decryptedLog0!.payload);

      // We check that the event was decoded correctly
      expect(event0?.value0).toStrictEqual(preimage[0]);
      expect(event0?.value1).toStrictEqual(preimage[1]);

      // We check that an event that does not match, is not decoded correctly due to an event type id mismatch
      const badEvent0 = TestLogContract.events.ExampleEvent1.decode(decryptedLog0!.payload);
      expect(badEvent0).toBe(undefined);

      const decryptedLog1 = TaggedLog.decryptAsIncoming(
        encryptedLogs[1],
        deriveMasterIncomingViewingSecretKey(wallets[0].getSecretKey()),
        L1EventPayload,
      );

      expect(decryptedLog1?.payload.contractAddress).toStrictEqual(testLogContract.address);
      expect(decryptedLog1?.payload.randomness).toStrictEqual(randomness[1]);
      expect(decryptedLog1?.payload.eventTypeId).toStrictEqual(
        new Fr(0x00000000000000000000000000000000000000000000000000000000d1be0447),
      );

      // We check our second event, which is a different type
      const event1 = TestLogContract.events.ExampleEvent1.decode(decryptedLog1!.payload);

      // We expect the fields to have been populated correctly
      expect(event1?.value2).toStrictEqual(preimage[2]);
      expect(event1?.value3).toStrictEqual(preimage[3]);

      // Again, trying to decode another event with mismatching data does not yield anything
      const badEvent1 = TestLogContract.events.ExampleEvent0.decode(decryptedLog1!.payload);
      expect(badEvent1).toBe(undefined);
    });
  });
});
