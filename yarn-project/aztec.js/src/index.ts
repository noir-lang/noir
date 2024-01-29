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
  WaitOpts,
  ContractFunctionInteraction,
  Contract,
  ContractBase,
  ContractMethod,
  SentTx,
  BatchCall,
  DeployMethod,
  DeploySentTx,
} from './contract/index.js';

export { ContractDeployer } from './contract_deployer/index.js';

export {
  generatePublicKey,
  FieldLike,
  EthAddressLike,
  computeMessageSecretHash,
  CheatCodes,
  AztecAddressLike,
  FunctionSelectorLike,
  isContractDeployed,
  EthCheatCodes,
  computeAuthWitMessageHash,
  waitForPXE,
  waitForAccountSynch,
} from './utils/index.js';

export { createPXEClient } from './rpc_clients/index.js';

export { AuthWitnessProvider } from './account/index.js';

export { AccountContract } from './account/index.js';
export { AccountManager } from './account_manager/index.js';

export { AccountWalletWithPrivateKey, AccountWallet, Wallet, SignerlessWallet } from './wallet/index.js';

// // TODO https://github.com/AztecProtocol/aztec-packages/issues/2632 --> FunctionSelector might not need to be exposed
// // here once the issue is resolved.
export {
  AztecAddress,
  EthAddress,
  Fr,
  Fq,
  GlobalVariables,
  GrumpkinScalar,
  Point,
  getContractDeploymentInfo,
} from '@aztec/circuits.js';

export { Grumpkin, Schnorr } from '@aztec/circuits.js/barretenberg';

export {
  AuthWitness,
  AztecNode,
  CompleteAddress,
  ContractData,
  DeployedContract,
  ExtendedContractData,
  ExtendedNote,
  FunctionCall,
  GrumpkinPrivateKey,
  INITIAL_L2_BLOCK_NUM,
  L2Actor,
  L2Block,
  L2BlockL2Logs,
  LogFilter,
  LogId,
  LogType,
  MerkleTreeId,
  Note,
  PXE,
  PackedArguments,
  PartialAddress,
  PublicKey,
  SyncStatus,
  Tx,
  TxExecutionRequest,
  TxHash,
  TxReceipt,
  TxStatus,
  UnencryptedL2Log,
  createAztecNodeClient,
  emptyFunctionCall,
  merkleTreeIds,
  mockTx,
  Comparator,
} from '@aztec/circuit-types';
export { NodeInfo } from '@aztec/types/interfaces';

// TODO: These kinds of things have no place on our public api.
// External devs will almost certainly have their own methods of doing these things.
// If we want to use them in our own "aztec.js consuming code", import them from foundation as needed.
export { encodeArguments } from '@aztec/foundation/abi';
export { sha256 } from '@aztec/foundation/crypto';
export { DebugLogger, createDebugLogger, onLog } from '@aztec/foundation/log';
export { retry, retryUntil } from '@aztec/foundation/retry';
export { sleep } from '@aztec/foundation/sleep';
export { elapsed } from '@aztec/foundation/timer';
export { fileURLToPath } from '@aztec/foundation/url';
export { to2Fields, toBigInt } from '@aztec/foundation/serialize';
export { toBigIntBE } from '@aztec/foundation/bigint-buffer';
export { makeFetch } from '@aztec/foundation/json-rpc/client';
export { FieldsOf } from '@aztec/foundation/types';

export {
  DeployL1Contracts,
  L1ContractArtifactsForDeployment,
  deployL1Contract,
  deployL1Contracts,
} from '@aztec/ethereum';

// Start of section that exports public api via granular api.
// Here you *can* do `export *` as the granular api defacto exports things explicitly.
// This entire index file will be deprecated at some point after we're satisfied.
export * from './api/init.js';
export * from './api/abi.js';
