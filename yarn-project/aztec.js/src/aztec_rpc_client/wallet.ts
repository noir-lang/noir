import { AztecAddress, Fr, PartialContractAddress, PrivateKey, PublicKey } from '@aztec/circuits.js';
import {
  AztecRPC,
  ContractData,
  ContractPublicData,
  DeployedContract,
  FunctionCall,
  L2BlockL2Logs,
  NodeInfo,
  SyncStatus,
  Tx,
  TxExecutionRequest,
  TxHash,
  TxReceipt,
} from '@aztec/types';

import { AccountImplementation, CreateTxRequestOpts } from '../account_impl/index.js';

/**
 * The wallet interface.
 */
export type Wallet = AccountImplementation & AztecRPC;

/**
 * A base class for Wallet implementations
 */
export abstract class BaseWallet implements Wallet {
  constructor(protected readonly rpc: AztecRPC) {}

  abstract getAddress(): AztecAddress;
  abstract createTxExecutionRequest(execs: FunctionCall[], opts?: CreateTxRequestOpts): Promise<TxExecutionRequest>;

  addAccount(privKey: PrivateKey, address: AztecAddress, partialContractAddress: Fr): Promise<AztecAddress> {
    return this.rpc.addAccount(privKey, address, partialContractAddress);
  }
  addPublicKeyAndPartialAddress(
    address: AztecAddress,
    publicKey: PublicKey,
    partialAddress: PartialContractAddress,
  ): Promise<void> {
    return this.rpc.addPublicKeyAndPartialAddress(address, publicKey, partialAddress);
  }
  getAccounts(): Promise<AztecAddress[]> {
    return this.rpc.getAccounts();
  }
  getPublicKey(address: AztecAddress): Promise<PublicKey> {
    return this.rpc.getPublicKey(address);
  }
  addContracts(contracts: DeployedContract[]): Promise<void> {
    return this.rpc.addContracts(contracts);
  }
  isContractDeployed(contract: AztecAddress): Promise<boolean> {
    return this.rpc.isContractDeployed(contract);
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
  getContractData(contractAddress: AztecAddress): Promise<ContractPublicData | undefined> {
    return this.rpc.getContractData(contractAddress);
  }
  getContractInfo(contractAddress: AztecAddress): Promise<ContractData | undefined> {
    return this.rpc.getContractInfo(contractAddress);
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
  getPublicKeyAndPartialAddress(address: AztecAddress): Promise<[PublicKey, PartialContractAddress]> {
    return this.rpc.getPublicKeyAndPartialAddress(address);
  }
  isSynchronised() {
    return this.rpc.isSynchronised();
  }
  isAccountSynchronised(account: AztecAddress) {
    return this.rpc.isAccountSynchronised(account);
  }
  getSyncStatus(): Promise<SyncStatus> {
    return this.rpc.getSyncStatus();
  }
}

/**
 * A simple wallet implementation that forwards authentication requests to a provided account implementation.
 */
export class AccountWallet extends BaseWallet {
  constructor(rpc: AztecRPC, protected accountImpl: AccountImplementation) {
    super(rpc);
  }
  getAddress(): AztecAddress {
    return this.accountImpl.getAddress();
  }
  createTxExecutionRequest(executions: FunctionCall[], opts: CreateTxRequestOpts = {}): Promise<TxExecutionRequest> {
    return this.accountImpl.createTxExecutionRequest(executions, opts);
  }
}
