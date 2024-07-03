import {
  AvmCircuitInputs,
  AztecAddress,
  ContractStorageRead,
  ContractStorageUpdateRequest,
  FunctionSelector,
  Gas,
  GlobalVariables,
  Header,
  L2ToL1Message,
  LogHash,
  MAX_L1_TO_L2_MSG_READ_REQUESTS_PER_CALL,
  MAX_L2_TO_L1_MSGS_PER_CALL,
  MAX_NOTE_HASHES_PER_CALL,
  MAX_NOTE_HASH_READ_REQUESTS_PER_CALL,
  MAX_NULLIFIERS_PER_CALL,
  MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL,
  MAX_NULLIFIER_READ_REQUESTS_PER_CALL,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
  MAX_PUBLIC_DATA_READS_PER_CALL,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
  MAX_UNENCRYPTED_LOGS_PER_CALL,
  NoteHash,
  Nullifier,
  PublicCircuitPublicInputs,
  ReadRequest,
  RevertCode,
} from '@aztec/circuits.js';
import { computeVarArgsHash } from '@aztec/circuits.js/hash';
import { padArrayEnd } from '@aztec/foundation/collection';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { AvmSimulator, type PublicContractsDB, type PublicExecutionResult, type PublicStateDB } from '@aztec/simulator';
import {
  getAvmTestContractBytecode,
  initContext,
  initExecutionEnvironment,
  initHostStorage,
  initPersistableStateManager,
} from '@aztec/simulator/avm/fixtures';

import { jest } from '@jest/globals';
import { mock } from 'jest-mock-extended';
import fs from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'path';

import { PublicSideEffectTrace } from '../../simulator/src/public/side_effect_trace.js';
import { SerializableContractInstance } from '../../types/src/contracts/contract_instance.js';
import { type BBSuccess, BB_RESULT, generateAvmProof, verifyAvmProof } from './bb/execute.js';
import { extractVkData } from './verification_key/verification_key_data.js';

const TIMEOUT = 60_000;
const TIMESTAMP = new Fr(99833);

describe('AVM WitGen, proof generation and verification', () => {
  const avmFunctionsAndCalldata: [string, Fr[]][] = [
    ['add_args_return', [new Fr(1), new Fr(2)]],
    ['get_address', []],
    ['note_hash_exists', [new Fr(1), new Fr(2)]],
    ['test_get_contract_instance', []],
    ['set_storage_single', [new Fr(1)]],
    ['set_storage_list', [new Fr(1), new Fr(2)]],
    ['read_storage_single', [new Fr(1)]],
    ['read_storage_list', [new Fr(1)]],
    ['new_note_hash', [new Fr(1)]],
    ['new_nullifier', [new Fr(1)]],
    ['nullifier_exists', [new Fr(1)]],
    ['l1_to_l2_msg_exists', [new Fr(1), new Fr(2)]],
    ['send_l2_to_l1_msg', [new Fr(1), new Fr(2)]],
    ['to_radix_le', [new Fr(10)]],
    ['nested_call_to_add', [new Fr(1), new Fr(2)]],
  ];

  it.each(avmFunctionsAndCalldata)(
    'Should prove %s',
    async (name, calldata) => {
      await proveAndVerifyAvmTestContract(name, calldata);
    },
    TIMEOUT,
  );

  /************************************************************************
   * Hashing functions
   ************************************************************************/
  describe('AVM hash functions', () => {
    const avmHashFunctions: [string, Fr[]][] = [
      [
        'keccak_hash',
        [
          new Fr(189),
          new Fr(0),
          new Fr(0),
          new Fr(0),
          new Fr(0),
          new Fr(0),
          new Fr(0),
          new Fr(0),
          new Fr(0),
          new Fr(0),
        ],
      ],
      [
        'poseidon2_hash',
        [new Fr(0), new Fr(1), new Fr(2), new Fr(3), new Fr(4), new Fr(5), new Fr(6), new Fr(7), new Fr(8), new Fr(9)],
      ],
      [
        'sha256_hash',
        [new Fr(0), new Fr(1), new Fr(2), new Fr(3), new Fr(4), new Fr(5), new Fr(6), new Fr(7), new Fr(8), new Fr(9)],
      ],
      [
        'pedersen_hash',
        [new Fr(0), new Fr(1), new Fr(2), new Fr(3), new Fr(4), new Fr(5), new Fr(6), new Fr(7), new Fr(8), new Fr(9)],
      ],
      [
        'pedersen_hash_with_index',
        [new Fr(0), new Fr(1), new Fr(2), new Fr(3), new Fr(4), new Fr(5), new Fr(6), new Fr(7), new Fr(8), new Fr(9)],
      ],
    ];

    it.each(avmHashFunctions)(
      'Should prove %s',
      async (name, calldata) => {
        await proveAndVerifyAvmTestContract(name, calldata);
      },
      TIMEOUT,
    );
  });

  it(
    'Should prove that timestamp matches',
    async () => {
      await proveAndVerifyAvmTestContract('assert_timestamp', [TIMESTAMP]);
    },
    TIMEOUT,
  );

  it(
    'Should prove that mutated timestamp does not match and a revert is performed',
    async () => {
      // The error assertion string must match with that of assert_timestamp noir function.
      await proveAndVerifyAvmTestContract('assert_timestamp', [TIMESTAMP.add(new Fr(1))], 'timestamp does not match');
    },
    TIMEOUT,
  );

  /************************************************************************
   * Avm Embedded Curve functions
   ************************************************************************/
  describe('AVM Embedded Curve functions', () => {
    const avmEmbeddedCurveFunctions: string[] = ['elliptic_curve_add_and_double', 'variable_base_msm'];
    it.each(avmEmbeddedCurveFunctions)(
      'Should prove %s',
      async name => {
        await proveAndVerifyAvmTestContract(name);
      },
      TIMEOUT,
    );
  });

  /************************************************************************
   * AvmContext functions
   ************************************************************************/
  describe('AVM Context functions', () => {
    const avmContextFunctions = [
      'get_address',
      'get_storage_address',
      'get_sender',
      'get_fee_per_l2_gas',
      'get_fee_per_da_gas',
      'get_transaction_fee',
      'get_function_selector',
      'get_chain_id',
      'get_version',
      'get_block_number',
      'get_timestamp',
      'get_l2_gas_left',
      'get_da_gas_left',
    ];

    it.each(avmContextFunctions)(
      'Should prove %s',
      async contextFunction => {
        await proveAndVerifyAvmTestContract(contextFunction);
      },
      TIMEOUT,
    );
  });
});

/************************************************************************
 * Helpers
 ************************************************************************/

/**
 * If assertionErrString is set, we expect a (non exceptional halting) revert due to a failing assertion and
 * we check that the revert reason error contains this string. However, the circuit must correctly prove the
 * execution.
 */
const proveAndVerifyAvmTestContract = async (
  functionName: string,
  calldata: Fr[] = [],
  assertionErrString?: string,
) => {
  const startSideEffectCounter = 0;
  const functionSelector = FunctionSelector.random();
  const globals = GlobalVariables.empty();
  globals.timestamp = TIMESTAMP;
  const environment = initExecutionEnvironment({ functionSelector, calldata, globals });

  const contractsDb = mock<PublicContractsDB>();
  const contractInstance = new SerializableContractInstance({
    version: 1,
    salt: new Fr(0x123),
    deployer: new Fr(0x456),
    contractClassId: new Fr(0x789),
    initializationHash: new Fr(0x101112),
    publicKeysHash: new Fr(0x161718),
  }).withAddress(environment.address);
  contractsDb.getContractInstance.mockResolvedValue(Promise.resolve(contractInstance));

  const storageDb = mock<PublicStateDB>();
  const storageValue = new Fr(5);
  storageDb.storageRead.mockResolvedValue(Promise.resolve(storageValue));

  const hostStorage = initHostStorage({ contractsDb });
  const trace = new PublicSideEffectTrace(startSideEffectCounter);
  const persistableState = initPersistableStateManager({ hostStorage, trace });
  const context = initContext({ env: environment, persistableState });
  const nestedCallBytecode = getAvmTestContractBytecode('add_args_return');
  jest.spyOn(hostStorage.contractsDb, 'getBytecode').mockResolvedValue(nestedCallBytecode);

  const startGas = new Gas(context.machineState.gasLeft.daGas, context.machineState.gasLeft.l2Gas);

  const internalLogger = createDebugLogger('aztec:avm-proving-test');
  const logger = (msg: string, _data?: any) => internalLogger.verbose(msg);

  // Use a simple contract that emits a side effect
  const bytecode = getAvmTestContractBytecode(functionName);
  // The paths for the barretenberg binary and the write path are hardcoded for now.
  const bbPath = path.resolve('../../barretenberg/cpp/build/bin/bb');
  const bbWorkingDirectory = await fs.mkdtemp(path.join(tmpdir(), 'bb-'));

  // First we simulate (though it's not needed in this simple case).
  const simulator = new AvmSimulator(context);
  const avmResult = await simulator.executeBytecode(bytecode);

  if (assertionErrString == undefined) {
    expect(avmResult.reverted).toBe(false);
  } else {
    // Explicit revert when an assertion failed.
    expect(avmResult.reverted).toBe(true);
    expect(avmResult.revertReason?.message).toContain(assertionErrString);
  }

  const pxResult = trace.toPublicExecutionResult(
    environment,
    startGas,
    /*endGasLeft=*/ Gas.from(context.machineState.gasLeft),
    /*bytecode=*/ simulator.getBytecode()!,
    avmResult,
    functionName,
  );

  const avmCircuitInputs = new AvmCircuitInputs(
    functionName,
    /*bytecode=*/ simulator.getBytecode()!, // uncompressed bytecode
    /*calldata=*/ context.environment.calldata,
    /*publicInputs=*/ getPublicInputs(pxResult),
    /*avmHints=*/ pxResult.avmCircuitHints,
  );

  // Then we prove.
  const proofRes = await generateAvmProof(bbPath, bbWorkingDirectory, avmCircuitInputs, logger);
  expect(proofRes.status).toEqual(BB_RESULT.SUCCESS);

  // Then we test VK extraction.
  const succeededRes = proofRes as BBSuccess;
  const verificationKey = await extractVkData(succeededRes.vkPath!);
  expect(verificationKey.keyAsBytes).toHaveLength(16);

  // Then we verify.
  const rawVkPath = path.join(succeededRes.vkPath!, 'vk');
  const verificationRes = await verifyAvmProof(bbPath, succeededRes.proofPath!, rawVkPath, logger);
  expect(verificationRes.status).toBe(BB_RESULT.SUCCESS);
};

// TODO: pub somewhere more usable - copied from abstract phase manager
const getPublicInputs = (result: PublicExecutionResult): PublicCircuitPublicInputs => {
  return PublicCircuitPublicInputs.from({
    callContext: result.executionRequest.callContext,
    proverAddress: AztecAddress.ZERO,
    argsHash: computeVarArgsHash(result.executionRequest.args),
    noteHashes: padArrayEnd(result.noteHashes, NoteHash.empty(), MAX_NOTE_HASHES_PER_CALL),
    nullifiers: padArrayEnd(result.nullifiers, Nullifier.empty(), MAX_NULLIFIERS_PER_CALL),
    l2ToL1Msgs: padArrayEnd(result.l2ToL1Messages, L2ToL1Message.empty(), MAX_L2_TO_L1_MSGS_PER_CALL),
    startSideEffectCounter: result.startSideEffectCounter,
    endSideEffectCounter: result.endSideEffectCounter,
    returnsHash: computeVarArgsHash(result.returnValues),
    noteHashReadRequests: padArrayEnd(
      result.noteHashReadRequests,
      ReadRequest.empty(),
      MAX_NOTE_HASH_READ_REQUESTS_PER_CALL,
    ),
    nullifierReadRequests: padArrayEnd(
      result.nullifierReadRequests,
      ReadRequest.empty(),
      MAX_NULLIFIER_READ_REQUESTS_PER_CALL,
    ),
    nullifierNonExistentReadRequests: padArrayEnd(
      result.nullifierNonExistentReadRequests,
      ReadRequest.empty(),
      MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL,
    ),
    l1ToL2MsgReadRequests: padArrayEnd(
      result.l1ToL2MsgReadRequests,
      ReadRequest.empty(),
      MAX_L1_TO_L2_MSG_READ_REQUESTS_PER_CALL,
    ),
    contractStorageReads: padArrayEnd(
      result.contractStorageReads,
      ContractStorageRead.empty(),
      MAX_PUBLIC_DATA_READS_PER_CALL,
    ),
    contractStorageUpdateRequests: padArrayEnd(
      result.contractStorageUpdateRequests,
      ContractStorageUpdateRequest.empty(),
      MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
    ),
    publicCallStackHashes: padArrayEnd([], Fr.zero(), MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL),
    unencryptedLogsHashes: padArrayEnd(result.unencryptedLogsHashes, LogHash.empty(), MAX_UNENCRYPTED_LOGS_PER_CALL),
    historicalHeader: Header.empty(),
    globalVariables: GlobalVariables.empty(),
    startGasLeft: Gas.from(result.startGasLeft),
    endGasLeft: Gas.from(result.endGasLeft),
    transactionFee: result.transactionFee,
    // TODO(@just-mitch): need better mapping from simulator to revert code.
    revertCode: result.reverted ? RevertCode.APP_LOGIC_REVERTED : RevertCode.OK,
  });
};
