import {
  AvmCircuitInputs,
  AztecAddress,
  ContractStorageRead,
  ContractStorageUpdateRequest,
  Gas,
  GlobalVariables,
  Header,
  L2ToL1Message,
  LogHash,
  MAX_L1_TO_L2_MSG_READ_REQUESTS_PER_CALL,
  MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
  MAX_NEW_NOTE_HASHES_PER_CALL,
  MAX_NEW_NULLIFIERS_PER_CALL,
  MAX_NOTE_HASH_READ_REQUESTS_PER_CALL,
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
import { AvmSimulator, type PublicExecutionResult } from '@aztec/simulator';
import { getAvmTestContractBytecode, initContext, initExecutionEnvironment } from '@aztec/simulator/avm/fixtures';

import fs from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'path';

import {
  convertAvmResultsToPxResult,
  createPublicExecution,
} from '../../simulator/src/public/transitional_adaptors.js';
import { type BBSuccess, BB_RESULT, generateAvmProof, verifyAvmProof } from './bb/execute.js';
import { extractVkData } from './verification_key/verification_key_data.js';

const TIMEOUT = 30_000;

describe('AVM WitGen, proof generation and verification', () => {
  it(
    'Should prove valid execution contract function that performs addition',
    async () => {
      await proveAndVerifyAvmTestContract('add_args_return', [new Fr(1), new Fr(2)]);
    },
    TIMEOUT,
  );

  it(
    'Should prove kernel get environment opcode',
    async () => {
      await proveAndVerifyAvmTestContract('get_address');
    },
    TIMEOUT,
  );

  it(
    'Should prove with note hash exists with hints',
    async () => {
      await proveAndVerifyAvmTestContract('note_hash_exists', [new Fr(1), new Fr(2)]);
    },
    TIMEOUT,
  );

  it(
    'Should prove new note hash',
    async () => {
      await proveAndVerifyAvmTestContract('new_note_hash', [new Fr(1)]);
    },
    TIMEOUT,
  );

  it(
    'Should prove new note hash',
    async () => {
      await proveAndVerifyAvmTestContract('new_note_hash', [new Fr(1)]);
    },
    TIMEOUT,
  );

  it(
    'Should prove new nullifier',
    async () => {
      await proveAndVerifyAvmTestContract('new_nullifier', [new Fr(1)]);
    },
    TIMEOUT,
  );

  it(
    'Should prove nullifier exists',
    async () => {
      await proveAndVerifyAvmTestContract('nullifier_exists', [new Fr(1)]);
    },
    TIMEOUT,
  );

  it(
    'Should prove l1 to l2 msg exists',
    async () => {
      await proveAndVerifyAvmTestContract('l1_to_l2_msg_exists', [new Fr(1), new Fr(2)]);
    },
    TIMEOUT,
  );

  it(
    'Should prove send l2 to l1 msg',
    async () => {
      await proveAndVerifyAvmTestContract('send_l2_to_l1_msg', [new Fr(1), new Fr(2)]);
    },
    TIMEOUT,
  );

  // TODO: requires revert
  // it("Should prove to radix",
  //   async () => {
  //     await proveAndVerifyAvmTestContract('to_radix_le', [new Fr(10)]);
  //   },
  //   TIMEOUT
  // )
});

const proveAndVerifyAvmTestContract = async (functionName: string, calldata: Fr[] = []) => {
  const startSideEffectCounter = 0;
  const environment = initExecutionEnvironment({ calldata });
  const context = initContext({ env: environment });

  const startGas = new Gas(context.machineState.gasLeft.daGas, context.machineState.gasLeft.l2Gas);
  const oldPublicExecution = createPublicExecution(startSideEffectCounter, environment, calldata);

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
  expect(avmResult.reverted).toBe(false);

  const pxResult = convertAvmResultsToPxResult(
    avmResult,
    startSideEffectCounter,
    oldPublicExecution,
    startGas,
    context,
    simulator.getBytecode(),
  );
  // TODO(dbanks12): public inputs should not be empty.... Need to construct them from AvmContext?
  const uncompressedBytecode = simulator.getBytecode()!;

  const publicInputs = getPublicInputs(pxResult);
  const avmCircuitInputs = new AvmCircuitInputs(
    uncompressedBytecode,
    context.environment.calldata,
    publicInputs,
    pxResult.avmHints,
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
    callContext: result.execution.callContext,
    proverAddress: AztecAddress.ZERO,
    argsHash: computeVarArgsHash(result.execution.args),
    newNoteHashes: padArrayEnd(result.newNoteHashes, NoteHash.empty(), MAX_NEW_NOTE_HASHES_PER_CALL),
    newNullifiers: padArrayEnd(result.newNullifiers, Nullifier.empty(), MAX_NEW_NULLIFIERS_PER_CALL),
    newL2ToL1Msgs: padArrayEnd(result.newL2ToL1Messages, L2ToL1Message.empty(), MAX_NEW_L2_TO_L1_MSGS_PER_CALL),
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
