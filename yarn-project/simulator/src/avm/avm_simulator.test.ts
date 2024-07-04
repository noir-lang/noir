import { GasFees } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { computeVarArgsHash } from '@aztec/circuits.js/hash';
import { FunctionSelector } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { keccak256, pedersenHash, poseidon2Hash, sha256 } from '@aztec/foundation/crypto';
import { Fq, Fr } from '@aztec/foundation/fields';
import { type Fieldable } from '@aztec/foundation/serialize';

import { randomInt } from 'crypto';
import { mock } from 'jest-mock-extended';

import { type PublicSideEffectTraceInterface } from '../public/side_effect_trace_interface.js';
import { type AvmContext } from './avm_context.js';
import { type AvmExecutionEnvironment } from './avm_execution_environment.js';
import { type MemoryValue, TypeTag, type Uint8 } from './avm_memory_types.js';
import { AvmSimulator } from './avm_simulator.js';
import { isAvmBytecode, markBytecodeAsAvm } from './bytecode_utils.js';
import {
  adjustCalldataIndex,
  getAvmTestContractBytecode,
  initContext,
  initExecutionEnvironment,
  initGlobalVariables,
  initHostStorage,
  initMachineState,
  initPersistableStateManager,
  randomMemoryBytes,
  randomMemoryFields,
} from './fixtures/index.js';
import { type HostStorage } from './journal/host_storage.js';
import { type AvmPersistableStateManager } from './journal/journal.js';
import { Add, CalldataCopy, Return } from './opcodes/index.js';
import { encodeToBytecode } from './serialization/bytecode_serialization.js';
import {
  mockGetBytecode,
  mockGetContractInstance,
  mockL1ToL2MessageExists,
  mockNoteHashExists,
  mockNullifierExists,
  mockStorageRead,
  mockStorageReadWithMap,
  mockTraceFork,
} from './test_utils.js';

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

  it('Should not be recognized as AVM bytecode (magic missing)', async () => {
    expect(!(await isAvmBytecode(bytecode)));
  });

  it('Should execute bytecode that performs basic addition', async () => {
    const context = initContext({ env: initExecutionEnvironment({ calldata }) });
    const { l2Gas: initialL2GasLeft } = context.machineState.gasLeft;
    const results = await new AvmSimulator(context).executeBytecode(markBytecodeAsAvm(bytecode));

    expect(results.reverted).toBe(false);
    expect(results.output).toEqual([new Fr(3)]);
    expect(context.machineState.l2GasLeft).toEqual(initialL2GasLeft - 30);
  });

  it('Should halt if runs out of gas', async () => {
    const context = initContext({
      env: initExecutionEnvironment({ calldata }),
      machineState: initMachineState({ l2GasLeft: 5 }),
    });

    const results = await new AvmSimulator(context).executeBytecode(markBytecodeAsAvm(bytecode));
    expect(results.reverted).toBe(true);
    expect(results.output).toEqual([]);
    expect(results.revertReason?.message).toEqual('Not enough L2GAS gas left');
    expect(context.machineState.l2GasLeft).toEqual(0);
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

  it('modulo and u1', async () => {
    const calldata: Fr[] = [new Fr(2)];
    const context = initContext({ env: initExecutionEnvironment({ calldata }) });

    const bytecode = getAvmTestContractBytecode('modulo2');
    const results = await new AvmSimulator(context).executeBytecode(bytecode);

    expect(results.reverted).toBe(false);
    expect(results.output).toEqual([new Fr(0)]);
  });

  it('Should be recognized as AVM bytecode (magic present)', async () => {
    const bytecode = getAvmTestContractBytecode('add_args_return');
    expect(await isAvmBytecode(bytecode));
  });

  it('elliptic curve operations', async () => {
    const context = initContext();

    const bytecode = getAvmTestContractBytecode('elliptic_curve_add_and_double');
    const results = await new AvmSimulator(context).executeBytecode(bytecode);

    expect(results.reverted).toBe(false);
    const grumpkin = new Grumpkin();
    const g3 = grumpkin.mul(grumpkin.generator(), new Fq(3));
    expect(results.output).toEqual([g3.x, g3.y, Fr.ZERO]);
  });

  it('variable msm operations', async () => {
    const context = initContext();

    const bytecode = getAvmTestContractBytecode('variable_base_msm');
    const results = await new AvmSimulator(context).executeBytecode(bytecode);

    expect(results.reverted).toBe(false);
    const grumpkin = new Grumpkin();
    const g3 = grumpkin.mul(grumpkin.generator(), new Fq(3));
    const g20 = grumpkin.mul(grumpkin.generator(), new Fq(20));
    const expectedResult = grumpkin.add(g3, g20);
    expect(results.output).toEqual([expectedResult.x, expectedResult.y, Fr.ZERO]);
  });

  describe('U128 addition and overflows', () => {
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

    it('Expect failure on U128::add() overflow', async () => {
      const bytecode = getAvmTestContractBytecode('u128_addition_overflow');
      const results = await new AvmSimulator(initContext()).executeBytecode(bytecode);
      expect(results.reverted).toBe(true);
      expect(results.revertReason?.message).toEqual('Assertion failed: attempt to add with overflow');
    });

    it('Expect failure on U128::from_integer() overflow', async () => {
      const bytecode = getAvmTestContractBytecode('u128_from_integer_overflow');
      const results = await new AvmSimulator(initContext()).executeBytecode(bytecode);
      expect(results.reverted).toBe(true);
      expect(results.revertReason?.message).toMatch('Assertion failed.');
      // Note: compiler intrinsic messages (like below) are not known to the AVM, they are recovered by the PXE.
      // "Assertion failed: call to assert_max_bit_size 'self.__assert_max_bit_size(bit_size)'"
    });
  });

  it('Logging', async () => {
    const context = initContext();
    const bytecode = getAvmTestContractBytecode('debug_logging');
    const results = await new AvmSimulator(context).executeBytecode(bytecode);

    expect(results.reverted).toBe(false);
    expect(results.output).toEqual([]);
  });

  it('Assertion message', async () => {
    const calldata: Fr[] = [new Fr(20)];
    const context = initContext({ env: initExecutionEnvironment({ calldata }) });

    const bytecode = getAvmTestContractBytecode('assert_nullifier_exists');
    const results = await new AvmSimulator(context).executeBytecode(bytecode);

    expect(results.reverted).toBe(true);
    expect(results.revertReason?.message).toEqual("Assertion failed: Nullifier doesn't exist!");
    expect(results.output).toEqual([
      new Fr(0),
      ...[..."Nullifier doesn't exist!"].flatMap(c => new Fr(c.charCodeAt(0))),
    ]);
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
    ['sha256_hash', /*input=*/ randomMemoryBytes(10), /*output=*/ sha256FromMemoryBytes],
    ['keccak_hash', /*input=*/ randomMemoryBytes(10), /*output=*/ keccak256FromMemoryBytes],
    ['poseidon2_hash', /*input=*/ randomMemoryFields(10), /*output=*/ poseidon2FromMemoryFields],
    ['pedersen_hash', /*input=*/ randomMemoryFields(10), /*output=*/ pedersenFromMemoryFields],
    ['pedersen_hash_with_index', /*input=*/ randomMemoryFields(10), /*output=*/ indexedPedersenFromMemoryFields],
  ])('Hashes in noir contracts', (name: string, input: MemoryValue[], output: (msg: any[]) => Fr[]) => {
    it(`Should execute contract function that performs ${name}`, async () => {
      const calldata = input.map(e => e.toFr());

      const context = initContext({ env: initExecutionEnvironment({ calldata }) });
      const bytecode = getAvmTestContractBytecode(name);
      const results = await new AvmSimulator(context).executeBytecode(bytecode);

      expect(results.reverted).toBe(false);
      expect(results.output).toEqual(output(input));
    });
  });

  describe('Environment getters', () => {
    const address = AztecAddress.random();
    const storageAddress = AztecAddress.random();
    const sender = AztecAddress.random();
    const functionSelector = FunctionSelector.random();
    const transactionFee = Fr.random();
    const chainId = Fr.random();
    const version = Fr.random();
    const blockNumber = Fr.random();
    const timestamp = new Fr(randomInt(100000)); // cap timestamp since must fit in u64
    const feePerDaGas = Fr.random();
    const feePerL2Gas = Fr.random();
    const gasFees = new GasFees(feePerDaGas, feePerL2Gas);
    const globals = initGlobalVariables({
      chainId,
      version,
      blockNumber,
      timestamp,
      gasFees,
    });
    const env = initExecutionEnvironment({
      address,
      storageAddress,
      sender,
      functionSelector,
      transactionFee,
      globals,
    });
    let context: AvmContext;
    beforeEach(() => {
      context = initContext({ env });
    });

    it.each([
      ['address', address.toField(), 'get_address'],
      ['storageAddress', storageAddress.toField(), 'get_storage_address'],
      ['sender', sender.toField(), 'get_sender'],
      ['functionSelector', functionSelector.toField(), 'get_function_selector'],
      ['transactionFee', transactionFee.toField(), 'get_transaction_fee'],
      ['chainId', chainId.toField(), 'get_chain_id'],
      ['version', version.toField(), 'get_version'],
      ['blockNumber', blockNumber.toField(), 'get_block_number'],
      ['timestamp', timestamp.toField(), 'get_timestamp'],
      ['feePerDaGas', feePerDaGas.toField(), 'get_fee_per_da_gas'],
      ['feePerL2Gas', feePerL2Gas.toField(), 'get_fee_per_l2_gas'],
    ])('%s getter', async (_name: string, value: Fr, functionName: string) => {
      const bytecode = getAvmTestContractBytecode(functionName);
      const results = await new AvmSimulator(context).executeBytecode(bytecode);

      expect(results.reverted).toBe(false);

      const returnData = results.output;
      expect(returnData).toEqual([value]);
    });
  });

  describe('AvmContextInputs', () => {
    it('selector', async () => {
      const context = initContext({
        env: initExecutionEnvironment({
          functionSelector: FunctionSelector.fromSignature('check_selector()'),
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

  it('conversions', async () => {
    const calldata: Fr[] = [new Fr(0b1011101010100)];
    const context = initContext({ env: initExecutionEnvironment({ calldata }) });

    const bytecode = getAvmTestContractBytecode('to_radix_le');
    const results = await new AvmSimulator(context).executeBytecode(bytecode);

    expect(results.reverted).toBe(false);
    const expectedResults = Buffer.concat('0010101011'.split('').map(c => new Fr(Number(c)).toBuffer()));
    const resultBuffer = Buffer.concat(results.output.map(f => f.toBuffer()));

    expect(resultBuffer.equals(expectedResults)).toBe(true);
  });

  describe('Side effects, world state, nested calls', () => {
    const address = new Fr(1);
    const storageAddress = new Fr(2);
    const sender = new Fr(42);
    const leafIndex = new Fr(7);
    const slotNumber = 1; // must update Noir contract if changing this
    const slot = new Fr(slotNumber);
    const listSlotNumber0 = 2; // must update Noir contract if changing this
    const listSlotNumber1 = listSlotNumber0 + 1;
    const listSlot0 = new Fr(listSlotNumber0);
    const listSlot1 = new Fr(listSlotNumber1);
    const value0 = new Fr(420);
    const value1 = new Fr(69);

    let hostStorage: HostStorage;
    let trace: PublicSideEffectTraceInterface;
    let persistableState: AvmPersistableStateManager;

    beforeEach(() => {
      hostStorage = initHostStorage();
      trace = mock<PublicSideEffectTraceInterface>();
      persistableState = initPersistableStateManager({ hostStorage, trace });
    });

    const createContext = (calldata: Fr[] = []) => {
      return initContext({
        persistableState,
        env: initExecutionEnvironment({ address, storageAddress, sender, calldata }),
      });
    };

    // Will check existence at leafIndex, but nothing may be found there and/or something may be found at mockAtLeafIndex
    describe.each([
      [/*mockAtLeafIndex=*/ undefined], // doesn't exist at all
      [/*mockAtLeafIndex=*/ leafIndex], // should be found!
      [/*mockAtLeafIndex=*/ leafIndex.add(Fr.ONE)], // won't be found! (checking leafIndex+1, but it exists at leafIndex)
    ])('Note hash checks', (mockAtLeafIndex?: Fr) => {
      const expectFound = mockAtLeafIndex !== undefined && mockAtLeafIndex.equals(leafIndex);
      const existsElsewhere = mockAtLeafIndex !== undefined && !mockAtLeafIndex.equals(leafIndex);
      const existsStr = expectFound ? 'DOES exist' : 'does NOT exist';
      const foundAtStr = existsElsewhere
        ? `at leafIndex=${mockAtLeafIndex.toNumber()} (exists at leafIndex=${leafIndex.toNumber()})`
        : '';
      it(`Should return ${expectFound} (and be traced) when noteHash ${existsStr} ${foundAtStr}`, async () => {
        const calldata = [value0, leafIndex];
        const context = createContext(calldata);
        const bytecode = getAvmTestContractBytecode('note_hash_exists');
        if (mockAtLeafIndex !== undefined) {
          mockNoteHashExists(hostStorage, mockAtLeafIndex, value0);
        }

        const results = await new AvmSimulator(context).executeBytecode(bytecode);
        expect(results.reverted).toBe(false);
        expect(results.output).toEqual([expectFound ? Fr.ONE : Fr.ZERO]);

        expect(trace.traceNoteHashCheck).toHaveBeenCalledTimes(1);
        expect(trace.traceNoteHashCheck).toHaveBeenCalledWith(
          storageAddress,
          /*noteHash=*/ value0,
          leafIndex,
          /*exists=*/ expectFound,
        );
      });
    });

    describe.each([[/*exists=*/ false], [/*exists=*/ true]])('Nullifier checks', (exists: boolean) => {
      const existsStr = exists ? 'DOES exist' : 'does NOT exist';
      it(`Should return ${exists} (and be traced) when nullifier ${existsStr}`, async () => {
        const calldata = [value0];
        const context = createContext(calldata);
        const bytecode = getAvmTestContractBytecode('nullifier_exists');

        if (exists) {
          mockNullifierExists(hostStorage, leafIndex, value0);
        }

        const results = await new AvmSimulator(context).executeBytecode(bytecode);
        expect(results.reverted).toBe(false);
        expect(results.output).toEqual([exists ? Fr.ONE : Fr.ZERO]);

        expect(trace.traceNullifierCheck).toHaveBeenCalledTimes(1);
        const isPending = false;
        // leafIndex is returned from DB call for nullifiers, so it is absent on DB miss
        const tracedLeafIndex = exists && !isPending ? leafIndex : Fr.ZERO;
        expect(trace.traceNullifierCheck).toHaveBeenCalledWith(
          storageAddress,
          /*nullifier=*/ value0,
          tracedLeafIndex,
          exists,
          isPending,
        );
      });
    });

    // Will check existence at leafIndex, but nothing may be found there and/or something may be found at mockAtLeafIndex
    describe.each([
      [/*mockAtLeafIndex=*/ undefined], // doesn't exist at all
      [/*mockAtLeafIndex=*/ leafIndex], // should be found!
      [/*mockAtLeafIndex=*/ leafIndex.add(Fr.ONE)], // won't be found! (checking leafIndex+1, but it exists at leafIndex)
    ])('L1ToL2 message checks', (mockAtLeafIndex?: Fr) => {
      const expectFound = mockAtLeafIndex !== undefined && mockAtLeafIndex.equals(leafIndex);
      const existsElsewhere = mockAtLeafIndex !== undefined && !mockAtLeafIndex.equals(leafIndex);
      const existsStr = expectFound ? 'DOES exist' : 'does NOT exist';
      const foundAtStr = existsElsewhere
        ? `at leafIndex=${mockAtLeafIndex.toNumber()} (exists at leafIndex=${leafIndex.toNumber()})`
        : '';

      it(`Should return ${expectFound} (and be traced) when message ${existsStr} ${foundAtStr}`, async () => {
        const calldata = [value0, leafIndex];
        const context = createContext(calldata);
        const bytecode = getAvmTestContractBytecode('l1_to_l2_msg_exists');
        if (mockAtLeafIndex !== undefined) {
          mockL1ToL2MessageExists(hostStorage, mockAtLeafIndex, value0, /*valueAtOtherIndices=*/ value1);
        }

        const results = await new AvmSimulator(context).executeBytecode(bytecode);
        expect(results.reverted).toBe(false);
        expect(results.output).toEqual([expectFound ? Fr.ONE : Fr.ZERO]);

        expect(trace.traceL1ToL2MessageCheck).toHaveBeenCalledTimes(1);
        expect(trace.traceL1ToL2MessageCheck).toHaveBeenCalledWith(
          address,
          /*msgHash=*/ value0,
          leafIndex,
          /*exists=*/ expectFound,
        );
      });
    });

    it('Should append a new note hash correctly', async () => {
      const calldata = [value0];
      const context = createContext(calldata);
      const bytecode = getAvmTestContractBytecode('new_note_hash');

      const results = await new AvmSimulator(context).executeBytecode(bytecode);
      expect(results.reverted).toBe(false);
      expect(results.output).toEqual([]);

      expect(trace.traceNewNoteHash).toHaveBeenCalledTimes(1);
      expect(trace.traceNewNoteHash).toHaveBeenCalledWith(
        expect.objectContaining(storageAddress),
        /*noteHash=*/ value0,
      );
    });

    it('Should append a new nullifier correctly', async () => {
      const calldata = [value0];
      const context = createContext(calldata);
      const bytecode = getAvmTestContractBytecode('new_nullifier');

      const results = await new AvmSimulator(context).executeBytecode(bytecode);
      expect(results.reverted).toBe(false);
      expect(results.output).toEqual([]);

      expect(trace.traceNewNullifier).toHaveBeenCalledTimes(1);
      expect(trace.traceNewNullifier).toHaveBeenCalledWith(
        expect.objectContaining(storageAddress),
        /*nullifier=*/ value0,
      );
    });

    describe('Cached nullifiers', () => {
      it(`Emits a nullifier and checks its existence`, async () => {
        const calldata = [value0];

        const context = createContext(calldata);
        const bytecode = getAvmTestContractBytecode('emit_nullifier_and_check');

        const results = await new AvmSimulator(context).executeBytecode(bytecode);
        expect(results.reverted).toBe(false);

        // New nullifier and nullifier existence check should be traced
        expect(trace.traceNewNullifier).toHaveBeenCalledTimes(1);
        expect(trace.traceNewNullifier).toHaveBeenCalledWith(
          expect.objectContaining(storageAddress),
          /*nullifier=*/ value0,
        );
        expect(trace.traceNullifierCheck).toHaveBeenCalledTimes(1);
        // leafIndex is returned from DB call for nullifiers, so it is absent on DB miss
        expect(trace.traceNullifierCheck).toHaveBeenCalledWith(
          storageAddress,
          /*nullifier=*/ value0,
          /*leafIndex=*/ Fr.ZERO,
          /*exists=*/ true,
          /*isPending=*/ true,
        );
      });
      it(`Emits same nullifier twice (expect failure)`, async () => {
        const calldata = [value0];

        const context = createContext(calldata);
        const bytecode = getAvmTestContractBytecode('nullifier_collision');

        const results = await new AvmSimulator(context).executeBytecode(bytecode);
        expect(results.reverted).toBe(true);
        expect(results.revertReason?.message).toMatch(/Attempted to emit duplicate nullifier/);

        // Nullifier should be traced exactly once
        expect(trace.traceNewNullifier).toHaveBeenCalledTimes(1);
        expect(trace.traceNewNullifier).toHaveBeenCalledWith(
          expect.objectContaining(storageAddress),
          /*nullifier=*/ value0,
        );
      });
    });

    describe('Unencrypted Logs', () => {
      it(`Emit unencrypted logs (should be traced)`, async () => {
        const context = createContext();
        const bytecode = getAvmTestContractBytecode('emit_unencrypted_log');

        const results = await new AvmSimulator(context).executeBytecode(bytecode);
        expect(results.reverted).toBe(false);

        const expectedFields = [new Fr(10), new Fr(20), new Fr(30)];
        const expectedString = 'Hello, world!'.split('').map(c => new Fr(c.charCodeAt(0)));
        const expectedCompressedString = [
          '\0A long time ago, in a galaxy fa',
          '\0r far away...\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0',
        ].map(s => new Fr(Buffer.from(s)));

        expect(trace.traceUnencryptedLog).toHaveBeenCalledTimes(3);
        expect(trace.traceUnencryptedLog).toHaveBeenCalledWith(address, expectedFields);
        expect(trace.traceUnencryptedLog).toHaveBeenCalledWith(address, expectedString);
        expect(trace.traceUnencryptedLog).toHaveBeenCalledWith(address, expectedCompressedString);
      });
    });

    describe('Public storage accesses', () => {
      it('Should set value in storage (single)', async () => {
        const calldata = [value0];

        const context = createContext(calldata);
        const bytecode = getAvmTestContractBytecode('set_storage_single');

        const results = await new AvmSimulator(context).executeBytecode(bytecode);
        expect(results.reverted).toBe(false);

        expect(await context.persistableState.peekStorage(storageAddress, slot)).toEqual(value0);

        expect(trace.tracePublicStorageWrite).toHaveBeenCalledTimes(1);
        expect(trace.tracePublicStorageWrite).toHaveBeenCalledWith(storageAddress, slot, value0);
      });

      it('Should read value in storage (single)', async () => {
        const context = createContext();
        mockStorageRead(hostStorage, value0);

        const bytecode = getAvmTestContractBytecode('read_storage_single');

        const results = await new AvmSimulator(context).executeBytecode(bytecode);
        expect(results.reverted).toBe(false);
        expect(results.output).toEqual([value0]);

        expect(trace.tracePublicStorageRead).toHaveBeenCalledTimes(1);
        expect(trace.tracePublicStorageRead).toHaveBeenCalledWith(
          storageAddress,
          slot,
          value0,
          /*exists=*/ true,
          /*cached=*/ false,
        );
      });

      it('Should set and read a value from storage (single)', async () => {
        const calldata = [value0];

        const context = createContext(calldata);
        const bytecode = getAvmTestContractBytecode('set_read_storage_single');
        const results = await new AvmSimulator(context).executeBytecode(bytecode);

        expect(results.reverted).toBe(false);
        expect(results.output).toEqual([value0]);

        expect(trace.tracePublicStorageWrite).toHaveBeenCalledTimes(1);
        expect(trace.tracePublicStorageWrite).toHaveBeenCalledWith(storageAddress, slot, value0);
        expect(trace.tracePublicStorageRead).toHaveBeenCalledTimes(1);
        expect(trace.tracePublicStorageRead).toHaveBeenCalledWith(
          storageAddress,
          slot,
          value0,
          /*exists=*/ true,
          /*cached=*/ true,
        );
      });

      it('Should set a value in storage (list)', async () => {
        const calldata = [value0, value1];

        const context = createContext(calldata);
        const bytecode = getAvmTestContractBytecode('set_storage_list');

        const results = await new AvmSimulator(context).executeBytecode(bytecode);
        expect(results.reverted).toBe(false);

        expect(await context.persistableState.peekStorage(storageAddress, listSlot0)).toEqual(calldata[0]);
        expect(await context.persistableState.peekStorage(storageAddress, listSlot1)).toEqual(calldata[1]);

        expect(trace.tracePublicStorageWrite).toHaveBeenCalledTimes(2);
        expect(trace.tracePublicStorageWrite).toHaveBeenCalledWith(storageAddress, listSlot0, value0);
        expect(trace.tracePublicStorageWrite).toHaveBeenCalledWith(storageAddress, listSlot1, value1);
      });

      it('Should read a value in storage (list)', async () => {
        const context = createContext();
        const mockedStorage = new Map([
          [listSlot0.toBigInt(), value0],
          [listSlot1.toBigInt(), value1],
        ]);
        mockStorageReadWithMap(hostStorage, mockedStorage);

        const bytecode = getAvmTestContractBytecode('read_storage_list');

        const results = await new AvmSimulator(context).executeBytecode(bytecode);
        expect(results.reverted).toBe(false);
        expect(results.output).toEqual([value0, value1]);

        expect(trace.tracePublicStorageRead).toHaveBeenCalledWith(
          storageAddress,
          listSlot0,
          value0,
          /*exists=*/ true,
          /*cached=*/ false,
        );
        expect(trace.tracePublicStorageRead).toHaveBeenCalledWith(
          storageAddress,
          listSlot1,
          value1,
          /*exists=*/ true,
          /*cached=*/ false,
        );
      });

      it('Should set a value in storage (map)', async () => {
        const calldata = [storageAddress, value0];

        const context = createContext(calldata);
        const bytecode = getAvmTestContractBytecode('set_storage_map');

        const results = await new AvmSimulator(context).executeBytecode(bytecode);
        expect(results.reverted).toBe(false);

        // returns the storage slot for modified key
        const mapSlotNumber = results.output[0].toBigInt();
        const mapSlot = new Fr(mapSlotNumber);

        expect(await context.persistableState.peekStorage(storageAddress, mapSlot)).toEqual(value0);

        expect(trace.tracePublicStorageWrite).toHaveBeenCalledTimes(1);
        expect(trace.tracePublicStorageWrite).toHaveBeenCalledWith(storageAddress, mapSlot, value0);
      });

      it('Should read-add-set a value in storage (map)', async () => {
        const calldata = [storageAddress, value0];

        const context = createContext(calldata);
        const bytecode = getAvmTestContractBytecode('add_storage_map');

        const results = await new AvmSimulator(context).executeBytecode(bytecode);
        expect(results.reverted).toBe(false);

        // returns the storage slot for modified key
        const mapSlotNumber = results.output[0].toBigInt();
        const mapSlot = new Fr(mapSlotNumber);

        expect(await context.persistableState.peekStorage(storageAddress, mapSlot)).toEqual(value0);

        expect(trace.tracePublicStorageRead).toHaveBeenCalledTimes(1);
        expect(trace.tracePublicStorageRead).toHaveBeenCalledWith(
          storageAddress,
          mapSlot,
          Fr.ZERO,
          /*exists=*/ false,
          /*cached=*/ false,
        );
        expect(trace.tracePublicStorageWrite).toHaveBeenCalledTimes(1);
        expect(trace.tracePublicStorageWrite).toHaveBeenCalledWith(storageAddress, mapSlot, value0);
      });

      it('Should read value in storage (map)', async () => {
        const calldata = [storageAddress];

        const context = createContext(calldata);
        mockStorageRead(hostStorage, value0);
        const bytecode = getAvmTestContractBytecode('read_storage_map');

        const results = await new AvmSimulator(context).executeBytecode(bytecode);
        expect(results.reverted).toBe(false);
        expect(results.output).toEqual([value0]);

        expect(trace.tracePublicStorageRead).toHaveBeenCalledTimes(1);
        // slot is the result of a pedersen hash and is therefore not known in the test
        expect(trace.tracePublicStorageRead).toHaveBeenCalledWith(
          storageAddress,
          expect.anything(),
          value0,
          /*exists=*/ true,
          /*cached=*/ false,
        );
      });
    });

    describe('Contract Instance Retrieval', () => {
      it(`Can getContractInstance`, async () => {
        const context = createContext();
        // Contract instance must match noir
        const contractInstance = {
          address: AztecAddress.random(),
          version: 1 as const,
          salt: new Fr(0x123),
          deployer: AztecAddress.fromBigInt(0x456n),
          contractClassId: new Fr(0x789),
          initializationHash: new Fr(0x101112),
          publicKeysHash: new Fr(0x161718),
        };
        mockGetContractInstance(hostStorage, contractInstance);

        const bytecode = getAvmTestContractBytecode('test_get_contract_instance_raw');

        const results = await new AvmSimulator(context).executeBytecode(bytecode);
        expect(results.reverted).toBe(false);

        expect(trace.traceGetContractInstance).toHaveBeenCalledTimes(1);
        expect(trace.traceGetContractInstance).toHaveBeenCalledWith({ exists: true, ...contractInstance });
      });
    });

    describe('Nested external calls', () => {
      const expectTracedNestedCall = (
        environment: AvmExecutionEnvironment,
        nestedTrace: PublicSideEffectTraceInterface,
        isStaticCall: boolean = false,
      ) => {
        expect(trace.traceNestedCall).toHaveBeenCalledTimes(1);
        expect(trace.traceNestedCall).toHaveBeenCalledWith(
          /*nestedCallTrace=*/ nestedTrace,
          /*nestedEnvironment=*/ expect.objectContaining({
            sender: environment.address, // sender is top-level call
            contractCallDepth: new Fr(1), // top call is depth 0, nested is depth 1
            header: environment.header, // just confirming that nested env looks roughly right
            globals: environment.globals, // just confirming that nested env looks roughly right
            isStaticCall: isStaticCall,
            // TODO(7121): can't check calldata like this since it is modified on environment construction
            // with AvmContextInputs. These should eventually go away.
            //calldata: expect.arrayContaining(environment.calldata), // top-level call forwards args
          }),
          /*startGasLeft=*/ expect.anything(),
          /*endGasLeft=*/ expect.anything(),
          /*bytecode=*/ expect.anything(), //decompressBytecodeIfCompressed(addBytecode),
          /*avmCallResults=*/ expect.anything(), // we don't have the NESTED call's results to check
          /*functionName=*/ expect.anything(),
        );
      };

      it(`Nested call`, async () => {
        const calldata = [value0, value1];
        const context = createContext(calldata);
        const callBytecode = getAvmTestContractBytecode('nested_call_to_add');
        const addBytecode = getAvmTestContractBytecode('add_args_return');
        mockGetBytecode(hostStorage, addBytecode);
        const nestedTrace = mock<PublicSideEffectTraceInterface>();
        mockTraceFork(trace, nestedTrace);

        const results = await new AvmSimulator(context).executeBytecode(callBytecode);
        expect(results.reverted).toBe(false);
        expect(results.output).toEqual([value0.add(value1)]);

        expectTracedNestedCall(context.environment, nestedTrace);
      });

      it(`Nested static call`, async () => {
        const calldata = [value0, value1];
        const context = createContext(calldata);
        const callBytecode = getAvmTestContractBytecode('nested_static_call_to_add');
        const addBytecode = getAvmTestContractBytecode('add_args_return');
        mockGetBytecode(hostStorage, addBytecode);
        const nestedTrace = mock<PublicSideEffectTraceInterface>();
        mockTraceFork(trace, nestedTrace);

        const results = await new AvmSimulator(context).executeBytecode(callBytecode);
        expect(results.reverted).toBe(false);
        expect(results.output).toEqual([value0.add(value1)]);

        expectTracedNestedCall(context.environment, nestedTrace, /*isStaticCall=*/ true);
      });

      it(`Nested call with not enough gas (expect failure)`, async () => {
        const gas = [/*l2=*/ 5, /*da=*/ 10000].map(g => new Fr(g));
        const calldata: Fr[] = [value0, value1, ...gas];
        const context = createContext(calldata);
        const callBytecode = getAvmTestContractBytecode('nested_call_to_add_with_gas');
        const addBytecode = getAvmTestContractBytecode('add_args_return');
        mockGetBytecode(hostStorage, addBytecode);
        mockTraceFork(trace);

        const results = await new AvmSimulator(context).executeBytecode(callBytecode);
        // TODO(7141): change this once we don't force rethrowing of exceptions.
        // Outer frame should not revert, but inner should, so the forwarded return value is 0
        // expect(results.revertReason).toBeUndefined();
        // expect(results.reverted).toBe(false);
        expect(results.reverted).toBe(true);
        expect(results.revertReason?.message).toEqual('Not enough L2GAS gas left');

        // Nested call should NOT have been made and therefore should not be traced
        expect(trace.traceNestedCall).toHaveBeenCalledTimes(0);
      });

      it(`Nested static call which modifies storage (expect failure)`, async () => {
        const context = createContext();
        const callBytecode = getAvmTestContractBytecode('nested_static_call_to_set_storage');
        const nestedBytecode = getAvmTestContractBytecode('set_storage_single');
        mockGetBytecode(hostStorage, nestedBytecode);
        mockTraceFork(trace);

        const results = await new AvmSimulator(context).executeBytecode(callBytecode);

        expect(results.reverted).toBe(true); // The outer call should revert.
        expect(results.revertReason?.message).toEqual(
          'Static call cannot update the state, emit L2->L1 messages or generate logs',
        );

        // TODO(7141): external call doesn't recover from nested exception until
        // we support recoverability of reverts (here and in kernel)
        //expectTracedNestedCall(context.environment, results, nestedTrace, /*isStaticCall=*/true);

        // Nested call should NOT have been able to write storage
        expect(trace.tracePublicStorageWrite).toHaveBeenCalledTimes(0);
      });

      it(`Nested calls rethrow exceptions`, async () => {
        const calldata = [value0, value1];
        const context = createContext(calldata);
        const callBytecode = getAvmTestContractBytecode('nested_call_to_add');
        // We actually don't pass the function ADD, but it's ok because the signature is the same.
        const nestedBytecode = getAvmTestContractBytecode('assert_same');
        mockGetBytecode(hostStorage, nestedBytecode);

        const results = await new AvmSimulator(context).executeBytecode(callBytecode);
        expect(results.reverted).toBe(true); // The outer call should revert.
        expect(results.revertReason?.message).toEqual('Assertion failed: Values are not equal');
      });
    });
  });
});

function sha256FromMemoryBytes(bytes: Uint8[]): Fr[] {
  return [...sha256(Buffer.concat(bytes.map(b => b.toBuffer())))].map(b => new Fr(b));
}

function keccak256FromMemoryBytes(bytes: Uint8[]): Fr[] {
  return [...keccak256(Buffer.concat(bytes.map(b => b.toBuffer())))].map(b => new Fr(b));
}

function poseidon2FromMemoryFields(fields: Fieldable[]): Fr[] {
  return [poseidon2Hash(fields)];
}

function pedersenFromMemoryFields(fields: Fieldable[]): Fr[] {
  return [pedersenHash(fields)];
}

function indexedPedersenFromMemoryFields(fields: Fieldable[]): Fr[] {
  return [pedersenHash(fields, /*index=*/ 20)];
}
