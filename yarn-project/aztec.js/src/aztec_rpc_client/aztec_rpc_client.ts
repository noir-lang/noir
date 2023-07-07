import { AztecAddress, AztecRPC, EthAddress, Fr, Point, Tx, TxHash } from '@aztec/aztec-rpc';
import { createJsonRpcClient } from '@aztec/foundation/json-rpc';
import { ContractData, ContractDeploymentTx, ContractPublicData, TxExecutionRequest } from '@aztec/types';

/**
 * A dictionary of the Aztec-deployed L1 contracts.
 */
export type L1ContractAddresses = {
  /**
   * Address fo the main Aztec rollup contract.
   */
  rollup: EthAddress;
  /**
   * Address of the contract that emits events on public contract deployment.
   */
  contractDeploymentEmitter: EthAddress;
  /**
   * Address of the L1/L2 messaging inbox contract.
   */
  inbox: EthAddress;

  /**
   * Registry Address.
   */
  registry: EthAddress;
};

/**
 * string dictionary of aztec contract addresses that we receive over http.
 */
type L1ContractAddressesResp = {
  [K in keyof L1ContractAddresses]: string;
};

export const createAztecRpcClient = (url: string): AztecRPC =>
  createJsonRpcClient<AztecRPC>(
    url,
    {
      AztecAddress,
      TxExecutionRequest,
      ContractData,
      ContractPublicData,
      TxHash,
      EthAddress,
      Point,
      Fr,
    },
    { Tx, ContractDeploymentTx },
    false,
  );

export const getL1ContractAddresses = async (url: string): Promise<L1ContractAddresses> => {
  const reqUrl = new URL(`${url}/api/l1-contract-addresses`);
  const response = (await (await fetch(reqUrl.toString())).json()) as unknown as L1ContractAddressesResp;
  const result = Object.fromEntries(
    Object.entries(response).map(([key, value]) => [key, EthAddress.fromString(value)]),
  );
  return result as L1ContractAddresses;
};
