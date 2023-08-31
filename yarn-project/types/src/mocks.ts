import {
  CompleteAddress,
  EthAddress,
  MAX_NEW_CONTRACTS_PER_TX,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  Proof,
} from '@aztec/circuits.js';
import { makePrivateKernelPublicInputsFinal, makePublicCallRequest } from '@aztec/circuits.js/factories';
import { ContractAbi } from '@aztec/foundation/abi';
import { randomBytes } from '@aztec/foundation/crypto';
import { Tuple } from '@aztec/foundation/serialize';

import times from 'lodash.times';

import { DeployedContract, ExtendedContractData, FunctionL2Logs, TxL2Logs } from './index.js';
import { Tx } from './tx/index.js';

/**
 * Testing utility to create empty logs composed from a single empty log.
 */
export function makeEmptyLogs(): TxL2Logs {
  const functionLogs = [new FunctionL2Logs([Buffer.alloc(0)])];
  return new TxL2Logs(functionLogs);
}

export const mockTx = (seed = 1) => {
  return new Tx(
    makePrivateKernelPublicInputsFinal(seed),
    new Proof(Buffer.alloc(0)),
    TxL2Logs.random(8, 3), // 8 priv function invocations creating 3 encrypted logs each
    TxL2Logs.random(11, 2), // 8 priv + 3 pub function invocations creating 2 unencrypted logs each
    times(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, makePublicCallRequest),
    times(MAX_NEW_CONTRACTS_PER_TX, ExtendedContractData.random) as Tuple<
      ExtendedContractData,
      typeof MAX_NEW_CONTRACTS_PER_TX
    >,
  );
};

export const randomContractAbi = (): ContractAbi => ({
  name: randomBytes(4).toString('hex'),
  functions: [],
});

export const randomDeployedContract = async (): Promise<DeployedContract> => ({
  abi: randomContractAbi(),
  completeAddress: await CompleteAddress.random(),
  portalContract: EthAddress.random(),
});
