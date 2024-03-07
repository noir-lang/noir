import { UnencryptedL2Log } from '@aztec/circuit-types';
import { EventSelector } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { keccak, pedersenHash, poseidonHash, sha256 } from '@aztec/foundation/crypto';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { AvmTestContractArtifact } from '@aztec/noir-contracts.js';

import { jest } from '@jest/globals';
import { strict as assert } from 'assert';

import { TypeTag } from './avm_memory_types.js';
import { AvmSimulator } from './avm_simulator.js';
import {
  initContext,
  initExecutionEnvironment,
  initGlobalVariables,
  initL1ToL2MessageOracleInput,
} from './fixtures/index.js';
import { Add, CalldataCopy, Return } from './opcodes/index.js';
import { encodeToBytecode } from './serialization/bytecode_serialization.js';

function getAvmTestContractBytecode(functionName: string): Buffer {
  const artifact = AvmTestContractArtifact.functions.find(f => f.name === functionName)!;
  assert(
    !!artifact.bytecode,
    `No bytecode found for function ${functionName}. Try re-running bootstraph.sh on the repository root.`,
  );
  return Buffer.from(artifact.bytecode, 'base64');
}

describe('AVM simulator', () => {
  it('Should execute bytecode that performs basic addition', async () => {
    const calldata: Fr[] = [new Fr(1), new Fr(2)];

    // Construct bytecode
    const bytecode = encodeToBytecode([
      new CalldataCopy(/*indirect=*/ 0, /*cdOffset=*/ 0, /*copySize=*/ 2, /*dstOffset=*/ 0),
      new Add(/*indirect=*/ 0, TypeTag.FIELD, /*aOffset=*/ 0, /*bOffset=*/ 1, /*dstOffset=*/ 2),
      new Return(/*indirect=*/ 0, /*returnOffset=*/ 2, /*copySize=*/ 1),
    ]);

    const context = initContext({ env: initExecutionEnvironment({ calldata }) });
    const results = await new AvmSimulator(context).executeBytecode(bytecode);

    expect(results.reverted).toBe(false);
    expect(results.output).toEqual([new Fr(3)]);
  });

  describe('Transpiled Noir contracts', () => {
    it('Should execute contract function that performs addition', async () => {
      const calldata: Fr[] = [new Fr(1), new Fr(2)];
      const context = initContext({ env: initExecutionEnvironment({ calldata }) });

      const bytecode = getAvmTestContractBytecode('avm_addArgsReturn');
      const results = await new AvmSimulator(context).executeBytecode(bytecode);

      expect(results.reverted).toBe(false);
      expect(results.output).toEqual([new Fr(3)]);
    });

    describe.each([
      ['avm_setOpcodeUint8', 8n],
      // ['avm_setOpcodeUint16', 60000n],
      ['avm_setOpcodeUint32', 1n << 30n],
      ['avm_setOpcodeUint64', 1n << 60n],
      // ['avm_setOpcodeUint128', 1n << 120n],
      ['avm_setOpcodeSmallField', 200n],
    ])('Should execute contract SET functions', (name: string, res: bigint) => {
      it(`Should execute contract function '${name}'`, async () => {
        const context = initContext();
        const bytecode = getAvmTestContractBytecode(name);
        const results = await new AvmSimulator(context).executeBytecode(bytecode);

        expect(results.reverted).toBe(false);
        expect(results.output).toEqual([new Fr(res)]);
      });
    });

    describe.each([
      ['avm_sha256_hash', sha256],
      ['avm_keccak_hash', keccak],
    ])('Hashes with 2 fields returned in noir contracts', (name: string, hashFunction: (data: Buffer) => Buffer) => {
      it(`Should execute contract function that performs ${name} hash`, async () => {
        const calldata = [new Fr(1), new Fr(2), new Fr(3)];
        const hash = hashFunction(Buffer.concat(calldata.map(f => f.toBuffer())));

        const context = initContext({ env: initExecutionEnvironment({ calldata }) });
        const bytecode = getAvmTestContractBytecode(name);
        const results = await new AvmSimulator(context).executeBytecode(bytecode);

        expect(results.reverted).toBe(false);
        expect(results.output).toEqual([new Fr(hash.subarray(0, 16)), new Fr(hash.subarray(16, 32))]);
      });
    });

    describe.each([
      ['avm_poseidon_hash', poseidonHash],
      ['avm_pedersen_hash', pedersenHash],
    ])('Hashes with field returned in noir contracts', (name: string, hashFunction: (data: Buffer[]) => Fr) => {
      it(`Should execute contract function that performs ${name} hash`, async () => {
        const calldata = [new Fr(1), new Fr(2), new Fr(3)];
        const hash = hashFunction(calldata.map(f => f.toBuffer()));

        const context = initContext({ env: initExecutionEnvironment({ calldata }) });
        const bytecode = getAvmTestContractBytecode(name);
        const results = await new AvmSimulator(context).executeBytecode(bytecode);

        expect(results.reverted).toBe(false);
        expect(results.output).toEqual([new Fr(hash)]);
      });
    });

    describe('Test env getters from noir contract', () => {
      const testEnvGetter = async (valueName: string, value: any, functionName: string, globalVar: boolean = false) => {
        // Execute
        let overrides = {};
        if (globalVar === true) {
          const globals = initGlobalVariables({ [valueName]: value });
          overrides = { globals };
        } else {
          overrides = { [valueName]: value };
        }
        const context = initContext({ env: initExecutionEnvironment(overrides) });
        const bytecode = getAvmTestContractBytecode(functionName);
        const results = await new AvmSimulator(context).executeBytecode(bytecode);

        expect(results.reverted).toBe(false);

        const returnData = results.output;
        expect(returnData).toEqual([value.toField()]);
      };

      it('address', async () => {
        const address = AztecAddress.fromField(new Fr(1));
        await testEnvGetter('address', address, 'avm_getAddress');
      });

      it('storageAddress', async () => {
        const storageAddress = AztecAddress.fromField(new Fr(1));
        await testEnvGetter('storageAddress', storageAddress, 'avm_getStorageAddress');
      });

      it('sender', async () => {
        const sender = AztecAddress.fromField(new Fr(1));
        await testEnvGetter('sender', sender, 'avm_getSender');
      });

      it('origin', async () => {
        const origin = AztecAddress.fromField(new Fr(1));
        await testEnvGetter('origin', origin, 'avm_getOrigin');
      });

      it('portal', async () => {
        const portal = EthAddress.fromField(new Fr(1));
        await testEnvGetter('portal', portal, 'avm_getPortal');
      });

      it('getFeePerL1Gas', async () => {
        const fee = new Fr(1);
        await testEnvGetter('feePerL1Gas', fee, 'avm_getFeePerL1Gas');
      });

      it('getFeePerL2Gas', async () => {
        const fee = new Fr(1);
        await testEnvGetter('feePerL2Gas', fee, 'avm_getFeePerL2Gas');
      });

      it('getFeePerDaGas', async () => {
        const fee = new Fr(1);
        await testEnvGetter('feePerDaGas', fee, 'avm_getFeePerDaGas');
      });

      it('chainId', async () => {
        const chainId = new Fr(1);
        await testEnvGetter('chainId', chainId, 'avm_getChainId', /*globalVar=*/ true);
      });

      it('version', async () => {
        const version = new Fr(1);
        await testEnvGetter('version', version, 'avm_getVersion', /*globalVar=*/ true);
      });

      it('blockNumber', async () => {
        const blockNumber = new Fr(1);
        await testEnvGetter('blockNumber', blockNumber, 'avm_getBlockNumber', /*globalVar=*/ true);
      });

      it('timestamp', async () => {
        const timestamp = new Fr(1);
        await testEnvGetter('timestamp', timestamp, 'avm_getTimestamp', /*globalVar=*/ true);
      });
    });

    describe('Test tree access functions from noir contract (notes & nullifiers)', () => {
      it(`Should execute contract function that checks if a note hash exists (it does not)`, async () => {
        const noteHash = new Fr(42);
        const leafIndex = new Fr(7);
        const calldata = [noteHash, leafIndex];

        const context = initContext({ env: initExecutionEnvironment({ calldata }) });
        const bytecode = getAvmTestContractBytecode('avm_note_hash_exists');
        const results = await new AvmSimulator(context).executeBytecode(bytecode);

        expect(results.reverted).toBe(false);
        expect(results.output).toEqual([/*exists=false*/ new Fr(0)]);

        // Note hash existence check should be in trace
        const trace = context.persistableState.flush();
        expect(trace.noteHashChecks).toEqual([expect.objectContaining({ noteHash, leafIndex, exists: false })]);
      });

      it(`Should execute contract function that checks if a note hash exists (it does)`, async () => {
        const noteHash = new Fr(42);
        const leafIndex = new Fr(7);
        const calldata = [noteHash, leafIndex];

        const context = initContext({ env: initExecutionEnvironment({ calldata }) });
        // note hash exists!
        jest
          .spyOn(context.persistableState.hostStorage.commitmentsDb, 'getCommitmentIndex')
          .mockReturnValue(Promise.resolve(BigInt(7)));
        const bytecode = getAvmTestContractBytecode('avm_note_hash_exists');
        const results = await new AvmSimulator(context).executeBytecode(bytecode);

        expect(results.reverted).toBe(false);
        expect(results.output).toEqual([/*exists=true*/ new Fr(1)]);

        // Note hash existence check should be in trace
        const trace = context.persistableState.flush();
        expect(trace.noteHashChecks).toEqual([expect.objectContaining({ noteHash, leafIndex, exists: true })]);
      });

      it(`Should execute contract function to emit unencrypted logs (should be traced)`, async () => {
        const context = initContext();
        const bytecode = getAvmTestContractBytecode('avm_emit_unencrypted_log');
        const results = await new AvmSimulator(context).executeBytecode(bytecode);

        expect(results.reverted).toBe(false);

        const expectedFields = [new Fr(10), new Fr(20), new Fr(30)];
        const expectedString = 'Hello, world!'.split('').map(c => new Fr(c.charCodeAt(0)));
        // FIXME: Try this once Brillig codegen produces uniform bit sizes for LT
        // const expectedCompressedString = Buffer.from('Hello, world!');
        expect(context.persistableState.flush().newLogs).toEqual([
          new UnencryptedL2Log(
            context.environment.address,
            new EventSelector(5),
            Buffer.concat(expectedFields.map(f => f.toBuffer())),
          ),
          new UnencryptedL2Log(
            context.environment.address,
            new EventSelector(8),
            Buffer.concat(expectedString.map(f => f.toBuffer())),
          ),
          // new UnencryptedL2Log(
          //   context.environment.address,
          //   new EventSelector(10),
          //   expectedCompressedString,
          // ),
        ]);
      });

      it(`Should execute contract function to emit note hash (should be traced)`, async () => {
        const utxo = new Fr(42);
        const calldata = [utxo];

        const context = initContext({ env: initExecutionEnvironment({ calldata }) });
        const bytecode = getAvmTestContractBytecode('avm_new_note_hash');
        const results = await new AvmSimulator(context).executeBytecode(bytecode);

        expect(results.reverted).toBe(false);

        expect(context.persistableState.flush().newNoteHashes).toEqual([utxo]);
      });

      it(`Should execute contract function to emit nullifier (should be traced)`, async () => {
        const utxo = new Fr(42);
        const calldata = [utxo];

        const context = initContext({ env: initExecutionEnvironment({ calldata }) });
        const bytecode = getAvmTestContractBytecode('avm_new_nullifier');
        const results = await new AvmSimulator(context).executeBytecode(bytecode);

        expect(results.reverted).toBe(false);

        expect(context.persistableState.flush().newNullifiers).toEqual([utxo]);
      });

      it(`Should execute contract function that checks if a nullifier exists (it does not)`, async () => {
        const utxo = new Fr(42);
        const calldata = [utxo];

        const context = initContext({ env: initExecutionEnvironment({ calldata }) });
        const bytecode = getAvmTestContractBytecode('avm_nullifier_exists');
        const results = await new AvmSimulator(context).executeBytecode(bytecode);

        expect(results.reverted).toBe(false);
        expect(results.output).toEqual([/*exists=false*/ new Fr(0)]);

        // Nullifier existence check should be in trace
        const trace = context.persistableState.flush();
        expect(trace.nullifierChecks.length).toEqual(1);
        expect(trace.nullifierChecks[0].exists).toEqual(false);
      });

      it(`Should execute contract function that checks if a nullifier exists (it does)`, async () => {
        const utxo = new Fr(42);
        const calldata = [utxo];

        const context = initContext({ env: initExecutionEnvironment({ calldata }) });
        // nullifier exists!
        jest
          .spyOn(context.persistableState.hostStorage.commitmentsDb, 'getNullifierIndex')
          .mockReturnValue(Promise.resolve(BigInt(42)));
        const bytecode = getAvmTestContractBytecode('avm_nullifier_exists');
        const results = await new AvmSimulator(context).executeBytecode(bytecode);

        expect(results.reverted).toBe(false);
        expect(results.output).toEqual([/*exists=true*/ new Fr(1)]);

        // Nullifier existence check should be in trace
        const trace = context.persistableState.flush();
        expect(trace.nullifierChecks.length).toEqual(1);
        expect(trace.nullifierChecks[0].exists).toEqual(true);
      });

      it(`Should execute contract function that checks emits a nullifier and checks its existence`, async () => {
        const utxo = new Fr(42);
        const calldata = [utxo];

        const context = initContext({ env: initExecutionEnvironment({ calldata }) });
        const bytecode = getAvmTestContractBytecode('avm_emit_nullifier_and_check');
        const results = await new AvmSimulator(context).executeBytecode(bytecode);

        expect(results.reverted).toBe(false);
        // Nullifier existence check should be in trace
        const trace = context.persistableState.flush();
        expect(trace.newNullifiers).toEqual([utxo]);
        expect(trace.nullifierChecks.length).toEqual(1);
        expect(trace.nullifierChecks[0].exists).toEqual(true);
      });

      it(`Should execute contract function that emits same nullifier twice (should fail)`, async () => {
        const utxo = new Fr(42);
        const calldata = [utxo];

        const context = initContext({ env: initExecutionEnvironment({ calldata }) });
        const bytecode = getAvmTestContractBytecode('avm_nullifier_collision');
        const results = await new AvmSimulator(context).executeBytecode(bytecode);

        expect(results.reverted).toBe(true);
        // Only the first nullifier should be in the trace, second one failed to add
        expect(context.persistableState.flush().newNullifiers).toEqual([utxo]);
      });
    });

    describe('Test tree access functions from noir contract (l1ToL2 messages)', () => {
      it(`Should execute contract function that checks if a message exists (it does not)`, async () => {
        const msgHash = new Fr(42);
        const leafIndex = new Fr(24);
        const calldata = [msgHash, leafIndex];

        const context = initContext({ env: initExecutionEnvironment({ calldata }) });
        const bytecode = getAvmTestContractBytecode('avm_l1_to_l2_msg_exists');
        const results = await new AvmSimulator(context).executeBytecode(bytecode);

        expect(results.reverted).toBe(false);
        expect(results.output).toEqual([/*exists=false*/ new Fr(0)]);
        // Message existence check should be in trace
        const trace = context.persistableState.flush();
        expect(trace.l1ToL2MessageChecks.length).toEqual(1);
        expect(trace.l1ToL2MessageChecks[0].exists).toEqual(false);
      });

      it(`Should execute contract function that checks if a message exists (it does)`, async () => {
        const msgHash = new Fr(42);
        const leafIndex = new Fr(24);
        const calldata = [msgHash, leafIndex];

        const context = initContext({ env: initExecutionEnvironment({ calldata }) });
        jest
          .spyOn(context.persistableState.hostStorage.commitmentsDb, 'getL1ToL2MembershipWitness')
          .mockResolvedValue(initL1ToL2MessageOracleInput(leafIndex.toBigInt()));
        const bytecode = getAvmTestContractBytecode('avm_l1_to_l2_msg_exists');
        const results = await new AvmSimulator(context).executeBytecode(bytecode);

        expect(results.reverted).toBe(false);
        expect(results.output).toEqual([/*exists=false*/ new Fr(1)]);
        // Message existence check should be in trace
        const trace = context.persistableState.flush();
        expect(trace.l1ToL2MessageChecks.length).toEqual(1);
        expect(trace.l1ToL2MessageChecks[0].exists).toEqual(true);
      });
    });

    describe('Storage accesses', () => {
      it('Should set a single value in storage', async () => {
        // We are setting the owner
        const slot = 1n;
        const sender = AztecAddress.fromField(new Fr(1));
        const address = AztecAddress.fromField(new Fr(420));

        const context = initContext({
          env: initExecutionEnvironment({ sender, address, storageAddress: address }),
        });
        const bytecode = getAvmTestContractBytecode('avm_setAdmin');
        const results = await new AvmSimulator(context).executeBytecode(bytecode);

        // Get contract function artifact
        expect(results.reverted).toBe(false);

        // Contract 420 - Storage slot 1 should contain the value 1
        const worldState = context.persistableState.flush();

        const storageSlot = worldState.currentStorageValue.get(address.toBigInt())!;
        const adminSlotValue = storageSlot.get(slot);
        expect(adminSlotValue).toEqual(sender.toField());

        // Tracing
        const storageTrace = worldState.storageWrites.get(address.toBigInt())!;
        const slotTrace = storageTrace.get(slot);
        expect(slotTrace).toEqual([sender.toField()]);
      });

      it('Should read a value from storage', async () => {
        // We are setting the owner
        const sender = AztecAddress.fromField(new Fr(1));
        const address = AztecAddress.fromField(new Fr(420));

        const context = initContext({
          env: initExecutionEnvironment({ sender, address, storageAddress: address }),
        });
        const bytecode = getAvmTestContractBytecode('avm_setAndRead');
        const results = await new AvmSimulator(context).executeBytecode(bytecode);

        expect(results.reverted).toBe(false);

        expect(results.output).toEqual([sender.toField()]);

        const worldState = context.persistableState.flush();

        // Test read trace
        const storageReadTrace = worldState.storageReads.get(address.toBigInt())!;
        const slotReadTrace = storageReadTrace.get(1n);
        expect(slotReadTrace).toEqual([sender.toField()]);

        // Test write trace
        const storageWriteTrace = worldState.storageWrites.get(address.toBigInt())!;
        const slotWriteTrace = storageWriteTrace.get(1n);
        expect(slotWriteTrace).toEqual([sender.toField()]);
      });
    });
  });
});
