import {
  ExtendedUnencryptedL2Log,
  L2Block,
  L2BlockL2Logs,
  NullifierMembershipWitness,
  TxReceipt,
} from '@aztec/circuit-types';
import { EthAddress, Fr } from '@aztec/circuits.js';
import { createJsonRpcClient, makeFetch } from '@aztec/foundation/json-rpc/client';

import { ArchiveSource } from '../archiver/archiver.js';

export const createArchiverClient = (url: string, fetch = makeFetch([1, 2, 3], true)): ArchiveSource =>
  createJsonRpcClient<ArchiveSource>(
    url,
    {
      EthAddress,
      ExtendedUnencryptedL2Log,
      Fr,
      L2Block,
      L2BlockL2Logs,
    },
    { TxReceipt, NullifierMembershipWitness },
    false,
    'archiver',
    fetch,
  );
