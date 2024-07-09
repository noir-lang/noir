/**
 * This is our public api.
 * Do NOT "export * from ..." here.
 * Everything here should be explicit, to ensure we can clearly see everything we're exposing to the world.
 * If it's exposed, people will use it, and then we can't remove/change the api without breaking client code.
 * At the time of writing we overexpose things that should only be internal.
 *
 * TODO: Review and narrow scope of public api.
 * We should also consider exposing subsections of the api via package.json exports, like we do with foundation.
 * This can allow consumers to import much smaller parts of the library to incur less overhead.
 * It will also allow web bundlers do perform intelligent chunking of bundles etc.
 * Some work as been done on this within the api folder, providing the alternative import style of e.g.:
 * ```typescript
 *   import { TxHash } from '@aztec.js/tx_hash'
 *   import { type ContractArtifact, type FunctionArtifact, FunctionSelector } from '@aztec/aztec.js/abi';
 *   import { AztecAddress } from '@aztec/aztec.js/aztec_address';
 *   import { EthAddress } from '@aztec/aztec.js/eth_address';
 * ```
 *
 * TODO: Ultimately reimplement this mega exporter by mega exporting a granular api (then deprecate it).
 */
export {
  BatchCall,
  Contract,
  ContractBase,
  ContractFunctionInteraction,
  type ContractMethod,
  type ContractNotes,
  type ContractStorageLayout,
  DeployMethod,
  DeploySentTx,
  type SendMethodOptions,
  SentTx,
  type WaitOpts,
} from './contract/index.js';

export { ContractDeployer } from './deployment/index.js';

export {
  AztecAddressLike,
  CheatCodes,
  EthAddressLike,
  EthCheatCodes,
  EventSelectorLike,
  FieldLike,
  FunctionSelectorLike,
  WrappedFieldLike,
  computeAuthWitMessageHash,
  computeInnerAuthWitHashFromAction,
  computeInnerAuthWitHash,
  generatePublicKey,
  waitForAccountSynch,
  waitForPXE,
} from './utils/index.js';
export { NoteSelector } from '@aztec/foundation/abi';

export { createPXEClient } from './rpc_clients/index.js';

export { AuthWitnessProvider } from './account/index.js';

export { AccountContract } from './account/index.js';
export { AccountManager, DeployAccountOptions } from './account_manager/index.js';

export { AccountWallet, AccountWalletWithSecretKey, SignerlessWallet, Wallet } from './wallet/index.js';

// // TODO https://github.com/AztecProtocol/aztec-packages/issues/2632 --> FunctionSelector might not need to be exposed
// // here once the issue is resolved.
export {
  AztecAddress,
  EthAddress,
  Fq,
  Fr,
  GlobalVariables,
  GrumpkinScalar,
  INITIAL_L2_BLOCK_NUM,
  Point,
  getContractClassFromArtifact,
  getContractInstanceFromDeployParams,
} from '@aztec/circuits.js';

export { computeSecretHash } from '@aztec/circuits.js/hash';

export {
  computeAppNullifierSecretKey,
  deriveKeys,
  deriveMasterIncomingViewingSecretKey,
  deriveMasterNullifierSecretKey,
} from '@aztec/circuits.js/keys';

export { Grumpkin, Schnorr } from '@aztec/circuits.js/barretenberg';

export {
  AuthWitness,
  AztecNode,
  Body,
  Comparator,
  CompleteAddress,
  EncryptedL2BlockL2Logs,
  EncryptedLogHeader,
  EncryptedNoteLogIncomingBody,
  EncryptedLogOutgoingBody,
  EventType,
  ExtendedNote,
  FunctionCall,
  L1Actor,
  L1ToL2Message,
  L2Actor,
  L2Block,
  L2BlockL2Logs,
  EncryptedNoteL2BlockL2Logs,
  LogFilter,
  LogId,
  LogType,
  MerkleTreeId,
  Note,
  PXE,
  PackedValues,
  PartialAddress,
  PublicKey,
  SiblingPath,
  SyncStatus,
  Tx,
  TxExecutionRequest,
  TxHash,
  TxReceipt,
  TxStatus,
  UnencryptedL2BlockL2Logs,
  UnencryptedL2Log,
  createAztecNodeClient,
  merkleTreeIds,
  mockTx,
  TaggedLog,
  L1NotePayload,
  L1EventPayload,
} from '@aztec/circuit-types';
export { NodeInfo } from '@aztec/types/interfaces';

export { ContractClassWithId, ContractInstanceWithAddress } from '@aztec/types/contracts';

// TODO: These kinds of things have no place on our public api.
// External devs will almost certainly have their own methods of doing these things.
// If we want to use them in our own "aztec.js consuming code", import them from foundation as needed.
export { encodeArguments } from '@aztec/foundation/abi';
export { toBigIntBE } from '@aztec/foundation/bigint-buffer';
export { sha256 } from '@aztec/foundation/crypto';
export { makeFetch } from '@aztec/foundation/json-rpc/client';
export { DebugLogger, createDebugLogger, onLog } from '@aztec/foundation/log';
export { retry, retryUntil } from '@aztec/foundation/retry';
export { to2Fields, toBigInt } from '@aztec/foundation/serialize';
export { sleep } from '@aztec/foundation/sleep';
export { elapsed } from '@aztec/foundation/timer';
export { FieldsOf } from '@aztec/foundation/types';
export { fileURLToPath } from '@aztec/foundation/url';

export {
  DeployL1Contracts,
  L1ContractArtifactsForDeployment,
  deployL1Contract,
  deployL1Contracts,
} from '@aztec/ethereum';

// Start of section that exports public api via granular api.
// Here you *can* do `export *` as the granular api defacto exports things explicitly.
// This entire index file will be deprecated at some point after we're satisfied.
export * from './api/abi.js';
export * from './api/fee.js';
export * from './api/init.js';
