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
export { AztecAddress, EthAddress, Point, Fr, FunctionSelector, GrumpkinScalar } from '@aztec/circuits.js';
export {
  PXE,
  ContractData,
  ExtendedContractData as ExtendedContractData,
  DeployedContract,
  FunctionCall,
  L2BlockL2Logs,
  LogFilter,
  UnencryptedL2Log,
  NodeInfo,
  NotePreimage,
  PackedArguments,
  PublicKey,
  GrumpkinPrivateKey,
  SyncStatus,
  Tx,
  TxExecutionRequest,
  TxHash,
  TxReceipt,
  TxStatus,
  emptyFunctionCall,
} from '@aztec/types';

export { createDebugLogger, DebugLogger } from '@aztec/foundation/log';
export { fileURLToPath } from '@aztec/foundation/url';
export { sleep } from '@aztec/foundation/sleep';
