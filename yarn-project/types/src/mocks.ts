import { AztecAddress, EthAddress, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, Proof } from '@aztec/circuits.js';
import { makeKernelPublicInputs, makePublicCallRequest } from '@aztec/circuits.js/factories';
import { ContractAbi } from '@aztec/foundation/abi';
import { randomBytes } from '@aztec/foundation/crypto';

import times from 'lodash.times';

import { DeployedContract, EncodedContractFunction, FunctionL2Logs, TxL2Logs } from './index.js';
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
    makeKernelPublicInputs(seed),
    new Proof(Buffer.alloc(0)),
    TxL2Logs.random(8, 3), // 8 priv function invocations creating 3 encrypted logs each
    TxL2Logs.random(11, 2), // 8 priv + 3 pub function invocations creating 2 unencrypted logs each
    times(3, EncodedContractFunction.random),
    times(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, makePublicCallRequest),
  );
};

export const randomContractAbi = (): ContractAbi => ({
  name: randomBytes(4).toString('hex'),
  functions: [],
});

export const randomDeployedContract = (): DeployedContract => ({
  abi: randomContractAbi(),
  address: AztecAddress.random(),
  portalContract: EthAddress.random(),
});
