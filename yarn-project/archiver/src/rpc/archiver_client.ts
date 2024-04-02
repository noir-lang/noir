import {
  EncryptedL2BlockL2Logs,
  ExtendedUnencryptedL2Log,
  L2Block,
  NullifierMembershipWitness,
  TxReceipt,
  UnencryptedL2BlockL2Logs,
} from '@aztec/circuit-types';
import { EthAddress, Fr } from '@aztec/circuits.js';
import { createJsonRpcClient, makeFetch } from '@aztec/foundation/json-rpc/client';

import { type ArchiveSource } from '../archiver/archiver.js';

export const createArchiverClient = (url: string, fetch = makeFetch([1, 2, 3], true)): ArchiveSource =>
  createJsonRpcClient<ArchiveSource>(
    url,
    {
      EthAddress,
      ExtendedUnencryptedL2Log,
      Fr,
      L2Block,
      EncryptedL2BlockL2Logs,
      UnencryptedL2BlockL2Logs,
    },
    { TxReceipt, NullifierMembershipWitness },
    false,
    'archiver',
    fetch,
  ) as ArchiveSource;
