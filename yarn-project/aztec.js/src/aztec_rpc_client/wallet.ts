import { AztecAddress, CircuitsWasm, Fr, PartialAddress, PrivateKey, PublicKey, TxContext } from '@aztec/circuits.js';
import {
  AztecRPC,
  ContractData,
  ContractDataAndBytecode,
  DeployedContract,
  FunctionCall,
  L2BlockL2Logs,
  NodeInfo,
  PackedArguments,
  SyncStatus,
  Tx,
  TxExecutionRequest,
  TxHash,
  TxReceipt,
} from '@aztec/types';

import { CreateTxRequestOpts, Entrypoint } from '../account/entrypoint/index.js';
import { CompleteAddress } from '../index.js';

/**
 * The wallet interface.
 */
export type Wallet = Entrypoint & AztecRPC;

/**
 * A base class for Wallet implementations
 */
export abstract class BaseWallet implements Wallet {
  constructor(protected readonly rpc: AztecRPC) {}

  abstract createTxExecutionRequest(execs: FunctionCall[], opts?: CreateTxRequestOpts): Promise<TxExecutionRequest>;

  addAccount(privKey: PrivateKey, address: AztecAddress, partialAddress: Fr): Promise<AztecAddress> {
    return this.rpc.addAccount(privKey, address, partialAddress);
  }
  addPublicKeyAndPartialAddress(
    address: AztecAddress,
    publicKey: PublicKey,
    partialAddress: PartialAddress,
  ): Promise<void> {
    return this.rpc.addPublicKeyAndPartialAddress(address, publicKey, partialAddress);
  }
  getAccounts(): Promise<AztecAddress[]> {
    return this.rpc.getAccounts();
  }
  addContracts(contracts: DeployedContract[]): Promise<void> {
    return this.rpc.addContracts(contracts);
  }
  simulateTx(txRequest: TxExecutionRequest): Promise<Tx> {
    return this.rpc.simulateTx(txRequest);
  }
  sendTx(tx: Tx): Promise<TxHash> {
    return this.rpc.sendTx(tx);
  }
  getTxReceipt(txHash: TxHash): Promise<TxReceipt> {
    return this.rpc.getTxReceipt(txHash);
  }
  getPreimagesAt(contract: AztecAddress, storageSlot: Fr): Promise<any> {
    return this.rpc.getPreimagesAt(contract, storageSlot);
  }
  getPublicStorageAt(contract: AztecAddress, storageSlot: Fr): Promise<any> {
    return this.rpc.getPublicStorageAt(contract, storageSlot);
  }
  viewTx(functionName: string, args: any[], to: AztecAddress, from?: AztecAddress | undefined): Promise<any> {
    return this.rpc.viewTx(functionName, args, to, from);
  }
  getContractDataAndBytecode(contractAddress: AztecAddress): Promise<ContractDataAndBytecode | undefined> {
    return this.rpc.getContractDataAndBytecode(contractAddress);
  }
  getContractData(contractAddress: AztecAddress): Promise<ContractData | undefined> {
    return this.rpc.getContractData(contractAddress);
  }
  getUnencryptedLogs(from: number, limit: number): Promise<L2BlockL2Logs[]> {
    return this.rpc.getUnencryptedLogs(from, limit);
  }
  getBlockNum(): Promise<number> {
    return this.rpc.getBlockNum();
  }
  getNodeInfo(): Promise<NodeInfo> {
    return this.rpc.getNodeInfo();
  }
  getPublicKeyAndPartialAddress(address: AztecAddress): Promise<[PublicKey, PartialAddress]> {
    return this.rpc.getPublicKeyAndPartialAddress(address);
  }
  isGlobalStateSynchronised() {
    return this.rpc.isGlobalStateSynchronised();
  }
  isAccountStateSynchronised(account: AztecAddress) {
    return this.rpc.isAccountStateSynchronised(account);
  }
  getSyncStatus(): Promise<SyncStatus> {
    return this.rpc.getSyncStatus();
  }
}

/**
 * A simple wallet implementation that forwards authentication requests to a provided entrypoint implementation.
 */
export class EntrypointWallet extends BaseWallet {
  constructor(rpc: AztecRPC, protected accountImpl: Entrypoint) {
    super(rpc);
  }
  createTxExecutionRequest(executions: FunctionCall[], opts: CreateTxRequestOpts = {}): Promise<TxExecutionRequest> {
    return this.accountImpl.createTxExecutionRequest(executions, opts);
  }
}

/**
 * A wallet implementation that forwards authentication requests to a provided account.
 */
export class AccountWallet extends EntrypointWallet {
  constructor(rpc: AztecRPC, protected accountImpl: Entrypoint, protected address: CompleteAddress) {
    super(rpc, accountImpl);
  }

  /** Returns the complete address of the account that implements this wallet. */
  public getCompleteAddress() {
    return this.address;
  }
}

/**
 * Wallet implementation which creates a transaction request directly to the requested contract without any signing.
 */
export class SignerlessWallet extends BaseWallet {
  async createTxExecutionRequest(executions: FunctionCall[]): Promise<TxExecutionRequest> {
    if (executions.length !== 1) {
      throw new Error(`Unexpected number of executions. Expected 1, received ${executions.length})`);
    }
    const [execution] = executions;
    const wasm = await CircuitsWasm.get();
    const packedArguments = await PackedArguments.fromArgs(execution.args, wasm);
    const { chainId, version } = await this.rpc.getNodeInfo();
    const txContext = TxContext.empty(chainId, version);
    return Promise.resolve(
      new TxExecutionRequest(execution.to, execution.functionData, packedArguments.hash, txContext, [packedArguments]),
    );
  }
}
