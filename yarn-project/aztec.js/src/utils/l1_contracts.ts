import { EthAddress } from '@aztec/circuits.js';
import { retryUntil } from '@aztec/foundation/retry';

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
   * Address of the L1/L2 messaging outbox contract.
   */
  outbox: EthAddress;
  /**
   * Address of the decoder helper contract
   */
  decoderHelper?: EthAddress;

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

export const getL1ContractAddresses = async (url: string): Promise<L1ContractAddresses> => {
  const reqUrl = new URL(`${url}/api/l1-contract-addresses`);
  const response = await retryUntil(
    async () => {
      try {
        return (await (await fetch(reqUrl.toString())).json()) as unknown as L1ContractAddressesResp;
      } catch (err) {
        // do nothing
      }
    },
    'isSandboxReady',
    120,
    1,
  );
  const result = Object.fromEntries(
    Object.entries(response).map(([key, value]) => [key, EthAddress.fromString(value)]),
  );
  return result as L1ContractAddresses;
};
