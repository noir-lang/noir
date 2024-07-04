import {
  type AccountWalletWithSecretKey,
  type AztecNode,
  EventType,
  Fr,
  L1EventPayload,
  type PXE,
  TaggedLog,
} from '@aztec/aztec.js';
import { deriveMasterIncomingViewingSecretKey } from '@aztec/circuits.js';
import { EventSelector } from '@aztec/foundation/abi';
import { makeTuple } from '@aztec/foundation/array';
import { type Tuple } from '@aztec/foundation/serialize';
import { type ExampleEvent0, type ExampleEvent1, TestLogContract } from '@aztec/noir-contracts.js';

import { jest } from '@jest/globals';

import { publicDeployAccounts, setup } from './fixtures/utils.js';

const TIMEOUT = 120_000;

describe('Logs', () => {
  let testLogContract: TestLogContract;
  jest.setTimeout(TIMEOUT);

  let wallets: AccountWalletWithSecretKey[];
  let node: AztecNode;
  let pxe: PXE;

  let teardown: () => Promise<void>;

  beforeAll(async () => {
    ({ teardown, wallets, aztecNode: node, pxe } = await setup(2));

    await publicDeployAccounts(wallets[0], wallets.slice(0, 2));

    testLogContract = await TestLogContract.deploy(wallets[0]).send().deployed();

    await pxe.registerRecipient(wallets[1].getCompleteAddress());
  });

  afterAll(() => teardown());

  describe('functionality around emitting an encrypted log', () => {
    it('emits multiple events as encrypted logs and decodes them one manually', async () => {
      const randomness = makeTuple(2, Fr.random);
      const preimage = makeTuple(4, Fr.random);

      const tx = await testLogContract.methods
        .emit_encrypted_events(wallets[1].getAddress(), randomness, preimage)
        .send()
        .wait();

      const txEffect = await node.getTxEffect(tx.txHash);

      const encryptedLogs = txEffect!.encryptedLogs.unrollLogs();
      expect(encryptedLogs.length).toBe(3);

      const decryptedLog0 = TaggedLog.decryptAsIncoming(
        encryptedLogs[0],
        deriveMasterIncomingViewingSecretKey(wallets[0].getSecretKey()),
        L1EventPayload,
      );

      expect(decryptedLog0?.payload.contractAddress).toStrictEqual(testLogContract.address);
      expect(decryptedLog0?.payload.randomness).toStrictEqual(randomness[0]);
      expect(decryptedLog0?.payload.eventTypeId).toStrictEqual(
        EventSelector.fromField(new Fr(0x00000000000000000000000000000000000000000000000000000000aa533f60)),
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
        // We want to skip the second emitted log as it is irrelevant in this test.
        encryptedLogs[2],
        deriveMasterIncomingViewingSecretKey(wallets[0].getSecretKey()),
        L1EventPayload,
      );

      expect(decryptedLog1?.payload.contractAddress).toStrictEqual(testLogContract.address);
      expect(decryptedLog1?.payload.randomness).toStrictEqual(randomness[1]);
      expect(decryptedLog1?.payload.eventTypeId).toStrictEqual(
        EventSelector.fromField(new Fr(0x00000000000000000000000000000000000000000000000000000000d1be0447)),
      );

      // We check our second event, which is a different type
      const event1 = TestLogContract.events.ExampleEvent1.decode(decryptedLog1!.payload);

      // We expect the fields to have been populated correctly
      expect(event1?.value2).toStrictEqual(preimage[2]);
      // We get the last byte here because value3 is of type u8
      expect(event1?.value3).toStrictEqual(new Fr(preimage[3].toBuffer().subarray(31)));

      // Again, trying to decode another event with mismatching data does not yield anything
      const badEvent1 = TestLogContract.events.ExampleEvent0.decode(decryptedLog1!.payload);
      expect(badEvent1).toBe(undefined);
    });

    it('emits multiple events as encrypted logs and decodes them', async () => {
      const randomness = makeTuple(5, makeTuple.bind(undefined, 2, Fr.random)) as Tuple<Tuple<Fr, 2>, 5>;
      const preimage = makeTuple(5, makeTuple.bind(undefined, 4, Fr.random)) as Tuple<Tuple<Fr, 4>, 5>;

      let i = 0;
      const firstTx = await testLogContract.methods
        .emit_encrypted_events(wallets[1].getAddress(), randomness[i], preimage[i])
        .send()
        .wait();
      await Promise.all(
        [...new Array(3)].map(() =>
          testLogContract.methods
            .emit_encrypted_events(wallets[1].getAddress(), randomness[++i], preimage[i])
            .send()
            .wait(),
        ),
      );
      const lastTx = await testLogContract.methods
        .emit_encrypted_events(wallets[1].getAddress(), randomness[++i], preimage[i])
        .send()
        .wait();

      // We get all the events we can decrypt with either our incoming or outgoing viewing keys
      const collectedEvent0s = await wallets[0].getEvents(
        EventType.Encrypted,
        TestLogContract.events.ExampleEvent0,
        firstTx.blockNumber!,
        lastTx.blockNumber! - firstTx.blockNumber! + 1,
      );

      const collectedEvent0sWithIncoming = await wallets[0].getEvents(
        EventType.Encrypted,
        TestLogContract.events.ExampleEvent0,
        firstTx.blockNumber!,
        lastTx.blockNumber! - firstTx.blockNumber! + 1,
        // This function can be called specifying the viewing public keys associated with the encrypted event.
        [wallets[0].getCompleteAddress().publicKeys.masterIncomingViewingPublicKey],
      );

      const collectedEvent0sWithOutgoing = await wallets[0].getEvents(
        EventType.Encrypted,
        TestLogContract.events.ExampleEvent0,
        firstTx.blockNumber!,
        lastTx.blockNumber! - firstTx.blockNumber! + 1,
        [wallets[0].getCompleteAddress().publicKeys.masterOutgoingViewingPublicKey],
      );

      const collectedEvent1s = await wallets[0].getEvents(
        EventType.Encrypted,
        TestLogContract.events.ExampleEvent1,
        firstTx.blockNumber!,
        lastTx.blockNumber! - firstTx.blockNumber! + 1,
        [wallets[0].getCompleteAddress().publicKeys.masterIncomingViewingPublicKey],
      );

      expect(collectedEvent0sWithIncoming.length).toBe(5);
      expect(collectedEvent0sWithOutgoing.length).toBe(5);
      expect(collectedEvent0s.length).toBe(10);
      expect(collectedEvent1s.length).toBe(5);

      const emptyEvent1s = await wallets[0].getEvents(
        EventType.Encrypted,
        TestLogContract.events.ExampleEvent1,
        firstTx.blockNumber!,
        lastTx.blockNumber! - firstTx.blockNumber! + 1,
        [wallets[0].getCompleteAddress().publicKeys.masterOutgoingViewingPublicKey],
      );

      expect(emptyEvent1s.length).toBe(0);

      const exampleEvent0Sort = (a: ExampleEvent0, b: ExampleEvent0) => (a.value0 > b.value0 ? 1 : -1);
      expect(collectedEvent0sWithIncoming.sort(exampleEvent0Sort)).toStrictEqual(
        preimage.map(preimage => ({ value0: preimage[0], value1: preimage[1] })).sort(exampleEvent0Sort),
      );

      expect(collectedEvent0sWithOutgoing.sort(exampleEvent0Sort)).toStrictEqual(
        preimage.map(preimage => ({ value0: preimage[0], value1: preimage[1] })).sort(exampleEvent0Sort),
      );

      expect([...collectedEvent0sWithIncoming, ...collectedEvent0sWithOutgoing].sort(exampleEvent0Sort)).toStrictEqual(
        collectedEvent0s.sort(exampleEvent0Sort),
      );

      const exampleEvent1Sort = (a: ExampleEvent1, b: ExampleEvent1) => (a.value2 > b.value2 ? 1 : -1);
      expect(collectedEvent1s.sort(exampleEvent1Sort)).toStrictEqual(
        preimage
          .map(preimage => ({
            value2: preimage[2],
            // We get the last byte here because value3 is of type u8
            value3: new Fr(preimage[3].toBuffer().subarray(31)),
          }))
          .sort(exampleEvent1Sort),
      );
    });

    it('emits multiple events as unencrypted logs and decodes them', async () => {
      const preimage = makeTuple(5, makeTuple.bind(undefined, 4, Fr.random)) as Tuple<Tuple<Fr, 4>, 5>;

      let i = 0;
      const firstTx = await testLogContract.methods.emit_unencrypted_events(preimage[i]).send().wait();
      await Promise.all(
        [...new Array(3)].map(() => testLogContract.methods.emit_unencrypted_events(preimage[++i]).send().wait()),
      );
      const lastTx = await testLogContract.methods.emit_unencrypted_events(preimage[++i]).send().wait();

      const collectedEvent0s = await wallets[0].getEvents(
        EventType.Unencrypted,
        TestLogContract.events.ExampleEvent0,
        firstTx.blockNumber!,
        lastTx.blockNumber! - firstTx.blockNumber! + 1,
      );

      const collectedEvent1s = await wallets[0].getEvents(
        EventType.Unencrypted,
        TestLogContract.events.ExampleEvent1,
        firstTx.blockNumber!,
        lastTx.blockNumber! - firstTx.blockNumber! + 1,
      );

      expect(collectedEvent0s.length).toBe(5);
      expect(collectedEvent1s.length).toBe(5);

      const exampleEvent0Sort = (a: ExampleEvent0, b: ExampleEvent0) => (a.value0 > b.value0 ? 1 : -1);
      expect(collectedEvent0s.sort(exampleEvent0Sort)).toStrictEqual(
        preimage.map(preimage => ({ value0: preimage[0], value1: preimage[1] })).sort(exampleEvent0Sort),
      );

      const exampleEvent1Sort = (a: ExampleEvent1, b: ExampleEvent1) => (a.value2 > b.value2 ? 1 : -1);
      expect(collectedEvent1s.sort(exampleEvent1Sort)).toStrictEqual(
        preimage
          .map(preimage => ({
            value2: preimage[2],
            // We get the last byte here because value3 is of type u8
            value3: new Fr(preimage[3].toBuffer().subarray(31)),
          }))
          .sort(exampleEvent1Sort),
      );
    });
  });
});
