import { BlockHeader, FunctionSelector } from '@aztec/circuits.js';
import { EventSelector } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { createJsonRpcClient, defaultFetch } from '@aztec/foundation/json-rpc/client';
import { SiblingPath } from '@aztec/types/membership';

import { ContractData, ExtendedContractData } from '../../contract_data.js';
import { AztecNode } from '../../interfaces/index.js';
import { L1ToL2MessageAndIndex } from '../../l1_to_l2_message.js';
import { L2Block } from '../../l2_block.js';
import { L2Tx } from '../../l2_tx.js';
import { ExtendedUnencryptedL2Log, L2BlockL2Logs, LogId } from '../../logs/index.js';
import { Tx, TxHash } from '../../tx/index.js';

/**
 * Creates a JSON-RPC client to remotely talk to an Aztec Node.
 * @param url - The URL of the Aztec Node.
 * @param fetch - The fetch implementation to use.
 * @returns A JSON-RPC client of Aztec Node.
 */
export function createAztecNodeClient(url: string, fetch = defaultFetch): AztecNode {
  return createJsonRpcClient<AztecNode>(
    url,
    {
      AztecAddress,
      EthAddress,
      ExtendedContractData,
      ExtendedUnencryptedL2Log,
      ContractData,
      Fr,
      EventSelector,
      FunctionSelector,
      BlockHeader,
      L2Block,
      L2Tx,
      LogId,
      TxHash,
      SiblingPath,
      L1ToL2MessageAndIndex,
    },
    { Tx, L2BlockL2Logs },
    false,
    'node',
    fetch,
  );
}
