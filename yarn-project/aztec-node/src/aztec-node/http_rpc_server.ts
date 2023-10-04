import { HistoricBlockData } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { JsonRpcServer } from '@aztec/foundation/json-rpc/server';
import {
  AztecNode,
  ContractData,
  ExtendedContractData,
  L1ToL2MessageAndIndex,
  L2Block,
  L2BlockL2Logs,
  L2Tx,
  SiblingPath,
  Tx,
  TxHash,
} from '@aztec/types';

/**
 * Wrap an AztecNode instance with a JSON RPC HTTP server.
 * @param node - The AztecNode
 * @returns An JSON-RPC HTTP server
 */
export function createAztecNodeRpcServer(node: AztecNode) {
  const rpc = new JsonRpcServer(
    node,
    {
      AztecAddress,
      EthAddress,
      ExtendedContractData,
      ContractData,
      Fr,
      HistoricBlockData,
      L2Block,
      L2Tx,
      TxHash,
      SiblingPath,
      L1ToL2MessageAndIndex,
    },
    { Tx, L2BlockL2Logs },
    false,
    // disable methods not part of the AztecNode interface
    ['start', 'stop'],
  );
  return rpc;
}
