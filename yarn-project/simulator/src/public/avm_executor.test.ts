import {
  AztecAddress,
  CallContext,
  EthAddress,
  FunctionData,
  FunctionSelector,
  Gas,
  GasSettings,
  type Header,
} from '@aztec/circuits.js';
import { makeHeader } from '@aztec/circuits.js/testing';
import { randomInt } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { AvmTestContractArtifact } from '@aztec/noir-contracts.js';

import { type MockProxy, mock } from 'jest-mock-extended';

import { initContext, initExecutionEnvironment } from '../avm/fixtures/index.js';
import { type CommitmentsDB, type PublicContractsDB, type PublicStateDB } from './db.js';
import { type PublicExecution } from './execution.js';
import { PublicExecutor } from './executor.js';

describe('AVM WitGen and Proof Generation', () => {
  let publicState: MockProxy<PublicStateDB>;
  let publicContracts: MockProxy<PublicContractsDB>;
  let commitmentsDb: MockProxy<CommitmentsDB>;
  let header: Header;

  const callContext = CallContext.from({
    msgSender: AztecAddress.random(),
    storageContractAddress: AztecAddress.random(),
    portalContractAddress: EthAddress.random(),
    functionSelector: FunctionSelector.empty(),
    gasLeft: Gas.test(),
    isDelegateCall: false,
    isStaticCall: false,
    sideEffectCounter: 0,
    gasSettings: GasSettings.empty(),
    transactionFee: Fr.ZERO,
  });
  const contractAddress = AztecAddress.random();

  beforeEach(() => {
    publicState = mock<PublicStateDB>();
    publicContracts = mock<PublicContractsDB>();
    commitmentsDb = mock<CommitmentsDB>();

    header = makeHeader(randomInt(1000000));
  }, 10000);

  it(
    'Should prove valid execution contract function that performs addition',
    async () => {
      const addArtifact = AvmTestContractArtifact.functions.find(f => f.name === 'add_args_return')!;
      const bytecode = addArtifact.bytecode;
      publicContracts.getBytecode.mockResolvedValue(bytecode);

      const functionData = FunctionData.fromAbi(addArtifact);
      const args: Fr[] = [new Fr(99), new Fr(12)];
      // We call initContext here to load up a AvmExecutionEnvironment that prepends the calldata with the function selector
      // and the args hash. In reality, we should simulate here and get this from the output of the simulation call.
      // For now, the interfaces for the PublicExecutor don't quite line up, so we are doing this.
      const context = initContext({ env: initExecutionEnvironment({ calldata: args }) });
      const execution: PublicExecution = {
        contractAddress,
        functionData,
        args: context.environment.calldata,
        callContext,
      };
      const executor = new PublicExecutor(publicState, publicContracts, commitmentsDb, header);
      const [proof, vk] = await executor.getAvmProof(execution);
      const valid = await executor.verifyAvmProof(vk, proof);
      expect(valid).toBe(true);
    },
    60 * 1000,
  ); // 60 seconds should be enough to generate the proof with 16-bit range checks
});
