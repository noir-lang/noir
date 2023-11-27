export * from './contract/index.js';
export * from './contract_deployer/index.js';
export * from './utils/index.js';
export * from './pxe_client.js';
export * from './account/index.js';
export * from './contract_deployer/deploy_method.js';
export * from './sandbox/index.js';
export * from './wallet/index.js';

// TODO https://github.com/AztecProtocol/aztec-packages/issues/2632 --> FunctionSelector might not need to be exposed
// here once the issue is resolved.
export {
  AztecAddress,
  EthAddress,
  Point,
  Fr,
  FunctionSelector,
  GlobalVariables,
  GrumpkinScalar,
  getContractDeploymentInfo,
} from '@aztec/circuits.js';
export { Grumpkin, Schnorr } from '@aztec/circuits.js/barretenberg';

export {
  AuthWitness,
  AztecNode,
  ContractData,
  DeployedContract,
  ExtendedContractData,
  ExtendedNote,
  FunctionCall,
  INITIAL_L2_BLOCK_NUM,
  GrumpkinPrivateKey,
  L2Actor,
  L2Block,
  L2BlockL2Logs,
  LogFilter,
  LogType,
  MerkleTreeId,
  NodeInfo,
  Note,
  PackedArguments,
  PartialAddress,
  PublicKey,
  PXE,
  SyncStatus,
  Tx,
  TxExecutionRequest,
  TxHash,
  TxReceipt,
  TxStatus,
  UnencryptedL2Log,
  emptyFunctionCall,
  createAztecNodeClient,
  merkleTreeIds,
  mockTx,
} from '@aztec/types';

export { ContractArtifact } from '@aztec/foundation/abi';
export { DebugLogger, createDebugLogger, onLog } from '@aztec/foundation/log';
export { fileURLToPath } from '@aztec/foundation/url';
export { sleep } from '@aztec/foundation/sleep';
export { elapsed } from '@aztec/foundation/timer';
export { retry, retryUntil } from '@aztec/foundation/retry';
export * from '@aztec/foundation/crypto';
export { to2Fields, toBigInt } from '@aztec/foundation/serialize';
export { toBigIntBE } from '@aztec/foundation/bigint-buffer';

export {
  deployL1Contract,
  deployL1Contracts,
  DeployL1Contracts,
  L1ContractArtifactsForDeployment,
} from '@aztec/ethereum';
