import {
  ContractData,
  EncodedContractFunction,
  ExtendedContractData,
  ExtendedUnencryptedL2Log,
  L1ToL2Message,
  L2Block,
  L2BlockL2Logs,
} from '@aztec/circuit-types';
import { EthAddress, Fr } from '@aztec/circuits.js';
import { createJsonRpcClient, makeFetch } from '@aztec/foundation/json-rpc/client';

import { ArchiveSource } from '../index.js';

export const createArchiverClient = (url: string, fetch = makeFetch([1, 2, 3], true)): ArchiveSource =>
  createJsonRpcClient<ArchiveSource>(
    url,
    {
      ContractData,
      EncodedContractFunction,
      EthAddress,
      ExtendedContractData,
      ExtendedUnencryptedL2Log,
      Fr,
      L1ToL2Message,
      L2Block,
      L2BlockL2Logs,
    },
    {},
    false,
    'archiver',
    fetch,
  );
