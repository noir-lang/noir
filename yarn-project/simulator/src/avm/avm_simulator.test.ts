import { UnencryptedL2Log } from '@aztec/circuit-types';
import { computeVarArgsHash } from '@aztec/circuits.js/hash';
import { EventSelector, FunctionSelector } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { keccak, pedersenHash, poseidonHash, sha256 } from '@aztec/foundation/crypto';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { type Fieldable } from '@aztec/foundation/serialize';
import { AvmNestedCallsTestContractArtifact, AvmTestContractArtifact } from '@aztec/noir-contracts.js';

import { jest } from '@jest/globals';
import { strict as assert } from 'assert';

import { isAvmBytecode } from '../public/transitional_adaptors.js';
import { AvmMachineState } from './avm_machine_state.js';
import { TypeTag } from './avm_memory_types.js';
import { AvmSimulator } from './avm_simulator.js';
import {
  adjustCalldataIndex,
  initContext,
  initExecutionEnvironment,
  initGlobalVariables,
  initL1ToL2MessageOracleInput,
  initMachineState,
} from './fixtures/index.js';
import { Add, CalldataCopy, Return } from './opcodes/index.js';
import { encodeToBytecode } from './serialization/bytecode_serialization.js';

describe('AVM simulator: injected bytecode', () => {
  let calldata: Fr[];
  let bytecode: Buffer;

  beforeAll(() => {
    calldata = [new Fr(1), new Fr(2)];
    bytecode = encodeToBytecode([
      new CalldataCopy(/*indirect=*/ 0, /*cdOffset=*/ adjustCalldataIndex(0), /*copySize=*/ 2, /*dstOffset=*/ 0),
      new Add(/*indirect=*/ 0, TypeTag.FIELD, /*aOffset=*/ 0, /*bOffset=*/ 1, /*dstOffset=*/ 2),
      new Return(/*indirect=*/ 0, /*returnOffset=*/ 2, /*copySize=*/ 1),
    ]);
  });

  it('Should not be recognized as AVM bytecode (magic missing)', () => {
    expect(!isAvmBytecode(bytecode));
  });

  it('Should execute bytecode that performs basic addition', async () => {
    const context = initContext({ env: initExecutionEnvironment({ calldata }) });
    const { l2GasLeft: initialL2GasLeft } = AvmMachineState.fromState(context.machineState);
    const results = await new AvmSimulator(context).executeBytecode(bytecode);

    expect(results.reverted).toBe(false);
    expect(results.output).toEqual([new Fr(3)]);
    expect(context.machineState.l2GasLeft).toEqual(initialL2GasLeft - 680);
  });

  it('Should halt if runs out of gas', async () => {
    const context = initContext({
      env: initExecutionEnvironment({ calldata }),
      machineState: initMachineState({ l2GasLeft: 5 }),
    });

    const results = await new AvmSimulator(context).executeBytecode(bytecode);
    expect(results.reverted).toBe(true);
    expect(results.output).toEqual([]);
    expect(results.revertReason?.name).toEqual('OutOfGasError');
    expect(context.machineState.l2GasLeft).toEqual(0);
    expect(context.machineState.l1GasLeft).toEqual(0);
    expect(context.machineState.daGasLeft).toEqual(0);
  });
});

describe('AVM simulator: transpiled Noir contracts', () => {
  it('addition', async () => {
    const calldata: Fr[] = [new Fr(1), new Fr(2)];
    const context = initContext({ env: initExecutionEnvironment({ calldata }) });

    const bytecode = getAvmTestContractBytecode('add_args_return');
    const results = await new AvmSimulator(context).executeBytecode(bytecode);

    expect(results.reverted).toBe(false);
    expect(results.output).toEqual([new Fr(3)]);
  });

  it('Should be recognized as AVM bytecode (magic present)', () => {
    const bytecode = getAvmTestContractBytecode('add_args_return');
    expect(isAvmBytecode(bytecode));
  });

  it('U128 addition', async () => {
    const calldata: Fr[] = [
      // First U128
      new Fr(1),
      new Fr(2),
      // Second U128
      new Fr(3),
      new Fr(4),
    ];
    const context = initContext({ env: initExecutionEnvironment({ calldata }) });

    const bytecode = getAvmTestContractBytecode('add_u128');
    const results = await new AvmSimulator(context).executeBytecode(bytecode);

    expect(results.reverted).toBe(false);
    expect(results.output).toEqual([new Fr(4), new Fr(6)]);
  });

  describe.each([
    ['set_opcode_u8', 8n],
    ['set_opcode_u32', 1n << 30n],
    ['set_opcode_u64', 1n << 60n],
    ['set_opcode_small_field', 0x001234567890abcdef1234567890abcdefn],
    ['set_opcode_big_field', 0x991234567890abcdef1234567890abcdefn],
  ])('SET functions', (name: string, res: bigint) => {
    it(`function '${name}'`, async () => {
      const context = initContext();
      const bytecode = getAvmTestContractBytecode(name);
      const results = await new AvmSimulator(context).executeBytecode(bytecode);

      expect(results.reverted).toBe(false);
      expect(results.output).toEqual([new Fr(res)]);
    });
  });

  describe.each([
    ['sha256_hash', sha256],
    ['keccak_hash', keccak],
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
    ['poseidon_hash', poseidonHash],
    ['pedersen_hash', pedersenHash],
    ['pedersen_hash_with_index', (m: Fieldable[]) => pedersenHash(m, 20)],
  ])('Hashes with field returned in noir contracts', (name: string, hashFunction: (data: Fieldable[]) => Fr) => {
    it(`Should execute contract function that performs ${name} hash`, async () => {
      const calldata = [new Fr(1), new Fr(2), new Fr(3)];
      const hash = hashFunction(calldata);

      const context = initContext({ env: initExecutionEnvironment({ calldata }) });
      const bytecode = getAvmTestContractBytecode(name);
      const results = await new AvmSimulator(context).executeBytecode(bytecode);

      expect(results.reverted).toBe(false);
      expect(results.output).toEqual([new Fr(hash)]);
    });
  });

  describe('Environment getters', () => {
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
      await testEnvGetter('address', address, 'get_address');
    });

    it('storageAddress', async () => {
      const storageAddress = AztecAddress.fromField(new Fr(1));
      await testEnvGetter('storageAddress', storageAddress, 'get_storage_address');
    });

    it('sender', async () => {
      const sender = AztecAddress.fromField(new Fr(1));
      await testEnvGetter('sender', sender, 'get_sender');
    });

    it('origin', async () => {
      const origin = AztecAddress.fromField(new Fr(1));
      await testEnvGetter('origin', origin, 'get_origin');
    });

    it('portal', async () => {
      const portal = EthAddress.fromField(new Fr(1));
      await testEnvGetter('portal', portal, 'get_portal');
    });

    it('getFeePerL1Gas', async () => {
      const fee = new Fr(1);
      await testEnvGetter('feePerL1Gas', fee, 'get_fee_per_l1_gas');
    });

    it('getFeePerL2Gas', async () => {
      const fee = new Fr(1);
      await testEnvGetter('feePerL2Gas', fee, 'get_fee_per_l2_gas');
    });

    it('getFeePerDaGas', async () => {
      const fee = new Fr(1);
      await testEnvGetter('feePerDaGas', fee, 'get_fee_per_da_gas');
    });

    it('chainId', async () => {
      const chainId = new Fr(1);
      await testEnvGetter('chainId', chainId, 'get_chain_id', /*globalVar=*/ true);
    });

    it('version', async () => {
      const version = new Fr(1);
      await testEnvGetter('version', version, 'get_version', /*globalVar=*/ true);
    });

    it('blockNumber', async () => {
      const blockNumber = new Fr(1);
      await testEnvGetter('blockNumber', blockNumber, 'get_block_number', /*globalVar=*/ true);
    });

    it('timestamp', async () => {
      const timestamp = new Fr(1);
      await testEnvGetter('timestamp', timestamp, 'get_timestamp', /*globalVar=*/ true);
    });
  });

  describe('AvmContextInputs', () => {
    it('selector', async () => {
      const context = initContext({
        env: initExecutionEnvironment({
          temporaryFunctionSelector: FunctionSelector.fromSignature('check_selector()'),
        }),
      });
      const bytecode = getAvmTestContractBytecode('check_selector');
      const results = await new AvmSimulator(context).executeBytecode(bytecode);

      expect(results.reverted).toBe(false);
    });

    it('get_args_hash', async () => {
      const calldata = [new Fr(8), new Fr(1), new Fr(2), new Fr(3)];

      const context = initContext({ env: initExecutionEnvironment({ calldata }) });
      const bytecode = getAvmTestContractBytecode('get_args_hash');
      const results = await new AvmSimulator(context).executeBytecode(bytecode);

      expect(results.reverted).toBe(false);
      expect(results.output).toEqual([computeVarArgsHash(calldata)]);
    });
  });

  describe('Tree access (notes & nullifiers)', () => {
    it(`Note hash exists (it does not)`, async () => {
      const noteHash = new Fr(42);
      const leafIndex = new Fr(7);
      const calldata = [noteHash, leafIndex];

      const context = initContext({ env: initExecutionEnvironment({ calldata }) });
      const bytecode = getAvmTestContractBytecode('note_hash_exists');
      const results = await new AvmSimulator(context).executeBytecode(bytecode);

      expect(results.reverted).toBe(false);
      expect(results.output).toEqual([/*exists=false*/ new Fr(0)]);

      // Note hash existence check should be in trace
      const trace = context.persistableState.flush();
      expect(trace.noteHashChecks).toEqual([expect.objectContaining({ noteHash, leafIndex, exists: false })]);
    });

    it(`Note hash exists (it does)`, async () => {
      const noteHash = new Fr(42);
      const leafIndex = new Fr(7);
      const calldata = [noteHash, leafIndex];

      const context = initContext({ env: initExecutionEnvironment({ calldata }) });
      // note hash exists!
      jest
        .spyOn(context.persistableState.hostStorage.commitmentsDb, 'getCommitmentIndex')
        .mockReturnValue(Promise.resolve(BigInt(7)));
      const bytecode = getAvmTestContractBytecode('note_hash_exists');
      const results = await new AvmSimulator(context).executeBytecode(bytecode);

      expect(results.reverted).toBe(false);
      expect(results.output).toEqual([/*exists=true*/ new Fr(1)]);

      // Note hash existence check should be in trace
      const trace = context.persistableState.flush();
      expect(trace.noteHashChecks).toEqual([expect.objectContaining({ noteHash, leafIndex, exists: true })]);
    });

    it(`Emit unencrypted logs (should be traced)`, async () => {
      const context = initContext();
      const bytecode = getAvmTestContractBytecode('emit_unencrypted_log');
      const results = await new AvmSimulator(context).executeBytecode(bytecode);

      expect(results.reverted).toBe(false);

      const expectedFields = [new Fr(10), new Fr(20), new Fr(30)];
      const expectedString = 'Hello, world!'.split('').map(c => new Fr(c.charCodeAt(0)));
      const expectedCompressedString = Buffer.from(
        '\0A long time ago, in a galaxy fa' + '\0r far away...\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0',
      );
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
        new UnencryptedL2Log(context.environment.address, new EventSelector(10), expectedCompressedString),
      ]);
    });

    it(`Emit note hash (should be traced)`, async () => {
      const utxo = new Fr(42);
      const calldata = [utxo];

      const context = initContext({ env: initExecutionEnvironment({ calldata }) });
      const bytecode = getAvmTestContractBytecode('new_note_hash');
      const results = await new AvmSimulator(context).executeBytecode(bytecode);

      expect(results.reverted).toBe(false);

      expect(context.persistableState.flush().newNoteHashes).toEqual([
        expect.objectContaining({
          storageAddress: context.environment.storageAddress,
          noteHash: utxo,
        }),
      ]);
    });

    it(`Emit nullifier (should be traced)`, async () => {
      const utxo = new Fr(42);
      const calldata = [utxo];

      const context = initContext({ env: initExecutionEnvironment({ calldata }) });
      const bytecode = getAvmTestContractBytecode('new_nullifier');
      const results = await new AvmSimulator(context).executeBytecode(bytecode);

      expect(results.reverted).toBe(false);

      expect(context.persistableState.flush().newNullifiers).toEqual([
        expect.objectContaining({
          storageAddress: context.environment.storageAddress,
          nullifier: utxo,
        }),
      ]);
    });

    it(`Nullifier exists (it does not)`, async () => {
      const utxo = new Fr(42);
      const calldata = [utxo];

      const context = initContext({ env: initExecutionEnvironment({ calldata }) });
      const bytecode = getAvmTestContractBytecode('nullifier_exists');
      const results = await new AvmSimulator(context).executeBytecode(bytecode);

      expect(results.reverted).toBe(false);
      expect(results.output).toEqual([/*exists=false*/ new Fr(0)]);

      // Nullifier existence check should be in trace
      const trace = context.persistableState.flush();
      expect(trace.nullifierChecks).toEqual([
        expect.objectContaining({
          storageAddress: context.environment.storageAddress,
          nullifier: utxo,
          exists: false,
          counter: expect.any(Fr),
          isPending: false,
          leafIndex: expect.any(Fr),
        }),
      ]);
    });

    it(`Nullifier exists (it does)`, async () => {
      const utxo = new Fr(42);
      const calldata = [utxo];

      const context = initContext({ env: initExecutionEnvironment({ calldata }) });
      // nullifier exists!
      jest
        .spyOn(context.persistableState.hostStorage.commitmentsDb, 'getNullifierIndex')
        .mockReturnValue(Promise.resolve(BigInt(42)));
      const bytecode = getAvmTestContractBytecode('nullifier_exists');
      const results = await new AvmSimulator(context).executeBytecode(bytecode);

      expect(results.reverted).toBe(false);
      expect(results.output).toEqual([/*exists=true*/ new Fr(1)]);

      // Nullifier existence check should be in trace
      const trace = context.persistableState.flush();
      expect(trace.nullifierChecks).toEqual([
        expect.objectContaining({
          storageAddress: context.environment.storageAddress,
          nullifier: utxo,
          exists: true,
          counter: expect.any(Fr),
          isPending: false,
          leafIndex: expect.any(Fr),
        }),
      ]);
    });

    it(`Emits a nullifier and checks its existence`, async () => {
      const utxo = new Fr(42);
      const calldata = [utxo];

      const context = initContext({ env: initExecutionEnvironment({ calldata }) });
      const bytecode = getAvmTestContractBytecode('emit_nullifier_and_check');
      const results = await new AvmSimulator(context).executeBytecode(bytecode);

      expect(results.reverted).toBe(false);
      // Nullifier existence check should be in trace
      const trace = context.persistableState.flush();
      expect(trace.newNullifiers).toEqual([
        expect.objectContaining({
          storageAddress: context.environment.storageAddress,
          nullifier: utxo,
        }),
      ]);
      expect(trace.nullifierChecks).toEqual([
        expect.objectContaining({
          storageAddress: context.environment.storageAddress,
          nullifier: utxo,
          exists: true,
          counter: expect.any(Fr),
          isPending: true,
          leafIndex: expect.any(Fr),
        }),
      ]);
    });

    it(`Emits same nullifier twice (should fail)`, async () => {
      const utxo = new Fr(42);
      const calldata = [utxo];

      const context = initContext({ env: initExecutionEnvironment({ calldata }) });
      const bytecode = getAvmTestContractBytecode('nullifier_collision');
      const results = await new AvmSimulator(context).executeBytecode(bytecode);

      expect(results.reverted).toBe(true);
      // Only the first nullifier should be in the trace, second one failed to add
      expect(context.persistableState.flush().newNullifiers).toEqual([
        expect.objectContaining({
          storageAddress: context.environment.storageAddress,
          nullifier: utxo,
        }),
      ]);
    });
  });

  describe('Test tree access (l1ToL2 messages)', () => {
    it(`Message exists (it does not)`, async () => {
      const msgHash = new Fr(42);
      const leafIndex = new Fr(24);
      const calldata = [msgHash, leafIndex];

      const context = initContext({ env: initExecutionEnvironment({ calldata }) });
      const bytecode = getAvmTestContractBytecode('l1_to_l2_msg_exists');
      const results = await new AvmSimulator(context).executeBytecode(bytecode);

      expect(results.reverted).toBe(false);
      expect(results.output).toEqual([/*exists=false*/ new Fr(0)]);
      // Message existence check should be in trace
      const trace = context.persistableState.flush();
      expect(trace.l1ToL2MessageChecks.length).toEqual(1);
      expect(trace.l1ToL2MessageChecks[0].exists).toEqual(false);
    });

    it(`Message exists (it does)`, async () => {
      const msgHash = new Fr(42);
      const leafIndex = new Fr(24);
      const calldata = [msgHash, leafIndex];

      const context = initContext({ env: initExecutionEnvironment({ calldata }) });
      jest
        .spyOn(context.persistableState.hostStorage.commitmentsDb, 'getL1ToL2MembershipWitness')
        .mockResolvedValue(initL1ToL2MessageOracleInput(leafIndex.toBigInt()));
      const bytecode = getAvmTestContractBytecode('l1_to_l2_msg_exists');
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
    it('Should set value in storage (single)', async () => {
      const slot = 1n;
      const address = AztecAddress.fromField(new Fr(420));
      const value = new Fr(88);
      const calldata = [value];

      const context = initContext({
        env: initExecutionEnvironment({ calldata, address, storageAddress: address }),
      });
      const bytecode = getAvmTestContractBytecode('set_storage_single');
      const results = await new AvmSimulator(context).executeBytecode(bytecode);

      expect(results.reverted).toBe(false);

      // World state
      const worldState = context.persistableState.flush();
      const storageSlot = worldState.currentStorageValue.get(address.toBigInt())!;
      const adminSlotValue = storageSlot.get(slot);
      expect(adminSlotValue).toEqual(value);

      // Tracing
      expect(worldState.storageWrites).toEqual([
        expect.objectContaining({
          storageAddress: address,
          slot: new Fr(slot),
          value: value,
        }),
      ]);
    });

    it('Should read value in storage (single)', async () => {
      const slot = 1n;
      const value = new Fr(12345);
      const address = AztecAddress.fromField(new Fr(420));
      const storage = new Map([[slot, value]]);

      const context = initContext({
        env: initExecutionEnvironment({ storageAddress: address }),
      });
      jest
        .spyOn(context.persistableState.hostStorage.publicStateDb, 'storageRead')
        .mockImplementation((_address, slot) => Promise.resolve(storage.get(slot.toBigInt())!));
      const bytecode = getAvmTestContractBytecode('read_storage_single');
      const results = await new AvmSimulator(context).executeBytecode(bytecode);

      // Get contract function artifact
      expect(results.reverted).toBe(false);
      expect(results.output).toEqual([value]);

      // Tracing
      const worldState = context.persistableState.flush();
      expect(worldState.storageReads).toEqual([
        expect.objectContaining({
          storageAddress: address,
          slot: new Fr(slot),
          value: value,
          exists: true,
        }),
      ]);
    });

    it('Should set and read a value from storage (single)', async () => {
      const slot = 1n;
      const value = new Fr(12345);
      const address = AztecAddress.fromField(new Fr(420));
      const calldata = [value];

      const context = initContext({
        env: initExecutionEnvironment({ calldata, address, storageAddress: address }),
      });
      const bytecode = getAvmTestContractBytecode('set_read_storage_single');
      const results = await new AvmSimulator(context).executeBytecode(bytecode);

      expect(results.reverted).toBe(false);
      expect(results.output).toEqual([value]);

      // Test read trace
      const worldState = context.persistableState.flush();
      expect(worldState.storageReads).toEqual([
        expect.objectContaining({
          storageAddress: address,
          slot: new Fr(slot),
          value: value,
          exists: true,
        }),
      ]);
      expect(worldState.storageWrites).toEqual([
        expect.objectContaining({
          storageAddress: address,
          slot: new Fr(slot),
          value: value,
        }),
      ]);
    });

    it('Should set a value in storage (list)', async () => {
      const slot = 2n;
      const sender = AztecAddress.fromField(new Fr(1));
      const address = AztecAddress.fromField(new Fr(420));
      const calldata = [new Fr(1), new Fr(2)];

      const context = initContext({
        env: initExecutionEnvironment({ sender, address, calldata, storageAddress: address }),
      });
      const bytecode = getAvmTestContractBytecode('set_storage_list');
      const results = await new AvmSimulator(context).executeBytecode(bytecode);

      expect(results.reverted).toBe(false);

      const worldState = context.persistableState.flush();
      const storageSlot = worldState.currentStorageValue.get(address.toBigInt())!;
      expect(storageSlot.get(slot)).toEqual(calldata[0]);
      expect(storageSlot.get(slot + 1n)).toEqual(calldata[1]);

      // Tracing
      expect(worldState.storageWrites).toEqual([
        expect.objectContaining({
          storageAddress: address,
          slot: new Fr(slot),
          value: calldata[0],
        }),
        expect.objectContaining({
          storageAddress: address,
          slot: new Fr(slot + 1n),
          value: calldata[1],
        }),
      ]);
    });

    it('Should read a value in storage (list)', async () => {
      const slot = 2n;
      const address = AztecAddress.fromField(new Fr(420));
      const values = [new Fr(1), new Fr(2)];
      const storage = new Map([
        [slot, values[0]],
        [slot + 1n, values[1]],
      ]);

      const context = initContext({
        env: initExecutionEnvironment({ address, storageAddress: address }),
      });
      jest
        .spyOn(context.persistableState.hostStorage.publicStateDb, 'storageRead')
        .mockImplementation((_address, slot) => Promise.resolve(storage.get(slot.toBigInt())!));
      const bytecode = getAvmTestContractBytecode('read_storage_list');
      const results = await new AvmSimulator(context).executeBytecode(bytecode);

      expect(results.reverted).toBe(false);
      expect(results.output).toEqual(values);

      // Tracing
      const worldState = context.persistableState.flush();
      expect(worldState.storageReads).toEqual([
        expect.objectContaining({
          storageAddress: address,
          slot: new Fr(slot),
          value: values[0],
          exists: true,
        }),
        expect.objectContaining({
          storageAddress: address,
          slot: new Fr(slot + 1n),
          value: values[1],
          exists: true,
        }),
      ]);
    });

    it('Should set a value in storage (map)', async () => {
      const address = AztecAddress.fromField(new Fr(420));
      const value = new Fr(12345);
      const calldata = [address.toField(), value];

      const context = initContext({
        env: initExecutionEnvironment({ address, calldata, storageAddress: address }),
      });
      const bytecode = getAvmTestContractBytecode('set_storage_map');
      const results = await new AvmSimulator(context).executeBytecode(bytecode);

      expect(results.reverted).toBe(false);
      // returns the storage slot for modified key
      const slotNumber = results.output[0].toBigInt();

      const worldState = context.persistableState.flush();
      const storageSlot = worldState.currentStorageValue.get(address.toBigInt())!;
      expect(storageSlot.get(slotNumber)).toEqual(value);

      // Tracing
      expect(worldState.storageWrites).toEqual([
        expect.objectContaining({
          storageAddress: address,
          slot: new Fr(slotNumber),
          value: value,
        }),
      ]);
    });

    it('Should read-add-set a value in storage (map)', async () => {
      const address = AztecAddress.fromField(new Fr(420));
      const value = new Fr(12345);
      const calldata = [address.toField(), value];

      const context = initContext({
        env: initExecutionEnvironment({ address, calldata, storageAddress: address }),
      });
      const bytecode = getAvmTestContractBytecode('add_storage_map');
      const results = await new AvmSimulator(context).executeBytecode(bytecode);

      expect(results.reverted).toBe(false);
      // returns the storage slot for modified key
      const slotNumber = results.output[0].toBigInt();

      const worldState = context.persistableState.flush();
      const storageSlot = worldState.currentStorageValue.get(address.toBigInt())!;
      expect(storageSlot.get(slotNumber)).toEqual(value);

      // Tracing
      expect(worldState.storageReads).toEqual([
        expect.objectContaining({
          storageAddress: address,
          slot: new Fr(slotNumber),
          value: Fr.ZERO,
          exists: false,
        }),
      ]);
      expect(worldState.storageWrites).toEqual([
        expect.objectContaining({
          storageAddress: address,
          slot: new Fr(slotNumber),
          value: value,
        }),
      ]);
    });

    it('Should read value in storage (map)', async () => {
      const value = new Fr(12345);
      const address = AztecAddress.fromField(new Fr(420));
      const calldata = [address.toField()];

      const context = initContext({
        env: initExecutionEnvironment({ calldata, address, storageAddress: address }),
      });
      jest
        .spyOn(context.persistableState.hostStorage.publicStateDb, 'storageRead')
        .mockReturnValue(Promise.resolve(value));
      const bytecode = getAvmTestContractBytecode('read_storage_map');
      const results = await new AvmSimulator(context).executeBytecode(bytecode);

      // Get contract function artifact
      expect(results.reverted).toBe(false);
      expect(results.output).toEqual([value]);

      // Tracing
      const worldState = context.persistableState.flush();
      expect(worldState.storageReads).toEqual([
        expect.objectContaining({
          storageAddress: address,
          // slot depends on pedersen hash of key, etc.
          value: value,
          exists: true,
        }),
      ]);
    });
  });

  describe('Contract', () => {
    it(`GETCONTRACTINSTANCE deserializes correctly`, async () => {
      const context = initContext();
      const contractInstance = {
        address: AztecAddress.random(),
        version: 1 as const,
        salt: new Fr(0x123),
        deployer: AztecAddress.fromBigInt(0x456n),
        contractClassId: new Fr(0x789),
        initializationHash: new Fr(0x101112),
        portalContractAddress: EthAddress.fromField(new Fr(0x131415)),
        publicKeysHash: new Fr(0x161718),
      };

      jest
        .spyOn(context.persistableState.hostStorage.contractsDb, 'getContractInstance')
        .mockReturnValue(Promise.resolve(contractInstance));
      const bytecode = getAvmTestContractBytecode('test_get_contract_instance_raw');
      const results = await new AvmSimulator(context).executeBytecode(bytecode);

      expect(results.reverted).toBe(false);
    });
  });

  describe('Nested external calls', () => {
    it(`Nested call succeeds`, async () => {
      const calldata: Fr[] = [new Fr(1), new Fr(2)];
      const callBytecode = getAvmNestedCallsTestContractBytecode('raw_nested_call_to_add');
      const addBytecode = getAvmNestedCallsTestContractBytecode('add_args_return');
      const context = initContext({ env: initExecutionEnvironment({ calldata }) });
      jest
        .spyOn(context.persistableState.hostStorage.contractsDb, 'getBytecode')
        .mockReturnValue(Promise.resolve(addBytecode));

      const results = await new AvmSimulator(context).executeBytecode(callBytecode);

      expect(results.revertReason).toBeUndefined();
      expect(results.reverted).toBe(false);
      expect(results.output).toEqual([new Fr(3)]);
    });

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/5625): gas not plumbed through correctly in nested calls.
    // it(`Nested call with not enough gas`, async () => {
    //   const gas = [/*l1=*/ 10000, /*l2=*/ 20, /*da=*/ 10000].map(g => new Fr(g));
    //   const calldata: Fr[] = [new Fr(1), new Fr(2), ...gas];
    //   const callBytecode = getAvmNestedCallsTestContractBytecode('raw_nested_call_to_add_with_gas');
    //   const addBytecode = getAvmNestedCallsTestContractBytecode('add_args_return');
    //   const context = initContext({ env: initExecutionEnvironment({ calldata }) });
    //   jest
    //     .spyOn(context.persistableState.hostStorage.contractsDb, 'getBytecode')
    //     .mockReturnValue(Promise.resolve(addBytecode));

    //   const results = await new AvmSimulator(context).executeBytecode(callBytecode);

    //   // Outer frame should not revert, but inner should, so the forwarded return value is 0
    //   expect(results.revertReason).toBeUndefined();
    //   expect(results.reverted).toBe(false);
    //   expect(results.output).toEqual([new Fr(0)]);
    // });

    it(`Nested call through the old interface`, async () => {
      const calldata: Fr[] = [new Fr(1), new Fr(2)];
      const callBytecode = getAvmNestedCallsTestContractBytecode('nested_call_to_add');
      const addBytecode = getAvmNestedCallsTestContractBytecode('add_args_return');
      const context = initContext({ env: initExecutionEnvironment({ calldata }) });
      jest
        .spyOn(context.persistableState.hostStorage.contractsDb, 'getBytecode')
        .mockReturnValue(Promise.resolve(addBytecode));

      const results = await new AvmSimulator(context).executeBytecode(callBytecode);

      expect(results.reverted).toBe(false);
      expect(results.output).toEqual([new Fr(3)]);
    });

    it(`Nested static call`, async () => {
      const calldata: Fr[] = [new Fr(1), new Fr(2)];
      const callBytecode = getAvmNestedCallsTestContractBytecode('raw_nested_static_call_to_add');
      const addBytecode = getAvmNestedCallsTestContractBytecode('add_args_return');
      const context = initContext({ env: initExecutionEnvironment({ calldata }) });
      jest
        .spyOn(context.persistableState.hostStorage.contractsDb, 'getBytecode')
        .mockReturnValue(Promise.resolve(addBytecode));

      const results = await new AvmSimulator(context).executeBytecode(callBytecode);

      expect(results.reverted).toBe(false);
      expect(results.output).toEqual([/*result=*/ new Fr(3), /*success=*/ new Fr(1)]);
    });

    it(`Nested static call which modifies storage`, async () => {
      const callBytecode = getAvmNestedCallsTestContractBytecode('raw_nested_static_call_to_set_storage');
      const nestedBytecode = getAvmNestedCallsTestContractBytecode('set_storage_single');
      const context = initContext();
      jest
        .spyOn(context.persistableState.hostStorage.contractsDb, 'getBytecode')
        .mockReturnValue(Promise.resolve(nestedBytecode));

      const results = await new AvmSimulator(context).executeBytecode(callBytecode);

      expect(results.reverted).toBe(false); // The outer call should not revert.
      expect(results.output).toEqual([new Fr(0)]); // The inner call should have reverted.
    });

    it(`Nested static call (old interface)`, async () => {
      const calldata: Fr[] = [new Fr(1), new Fr(2)];
      const callBytecode = getAvmNestedCallsTestContractBytecode('nested_static_call_to_add');
      const addBytecode = getAvmNestedCallsTestContractBytecode('add_args_return');
      const context = initContext({ env: initExecutionEnvironment({ calldata }) });
      jest
        .spyOn(context.persistableState.hostStorage.contractsDb, 'getBytecode')
        .mockReturnValue(Promise.resolve(addBytecode));

      const results = await new AvmSimulator(context).executeBytecode(callBytecode);

      expect(results.reverted).toBe(false);
      expect(results.output).toEqual([/*result=*/ new Fr(3)]);
    });

    it(`Nested static call which modifies storage (old interface)`, async () => {
      const callBytecode = getAvmNestedCallsTestContractBytecode('nested_static_call_to_set_storage');
      const nestedBytecode = getAvmNestedCallsTestContractBytecode('set_storage_single');
      const context = initContext();
      jest
        .spyOn(context.persistableState.hostStorage.contractsDb, 'getBytecode')
        .mockReturnValue(Promise.resolve(nestedBytecode));

      const results = await new AvmSimulator(context).executeBytecode(callBytecode);

      expect(results.reverted).toBe(true); // The outer call should revert.
      // TODO(fcarreiro): revertReason lost in translation between results.
      // expect(results.revertReason).toEqual(/StaticCallStorageAlterError/);
    });
  });
});

function getAvmTestContractBytecode(functionName: string): Buffer {
  const artifact = AvmTestContractArtifact.functions.find(f => f.name === functionName)!;
  assert(
    !!artifact?.bytecode,
    `No bytecode found for function ${functionName}. Try re-running bootstrap.sh on the repository root.`,
  );
  return artifact.bytecode;
}

function getAvmNestedCallsTestContractBytecode(functionName: string): Buffer {
  const artifact = AvmNestedCallsTestContractArtifact.functions.find(f => f.name === functionName)!;
  assert(
    !!artifact?.bytecode,
    `No bytecode found for function ${functionName}. Try re-running bootstrap.sh on the repository root.`,
  );
  return artifact.bytecode;
}
