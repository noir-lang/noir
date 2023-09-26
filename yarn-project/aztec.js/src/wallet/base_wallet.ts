import { AztecAddress, Fr, GrumpkinPrivateKey, PartialAddress } from '@aztec/circuits.js';
import {
  AuthWitness,
  AztecRPC,
  ContractData,
  DeployedContract,
  ExtendedContractData,
  FunctionCall,
  L2BlockL2Logs,
  L2Tx,
  NodeInfo,
  NotePreimage,
  SyncStatus,
  Tx,
  TxExecutionRequest,
  TxHash,
  TxReceipt,
} from '@aztec/types';

import { CompleteAddress } from '../index.js';
import { Wallet } from './index.js';

/**
 * A base class for Wallet implementations
 */
export abstract class BaseWallet implements Wallet {
  constructor(protected readonly rpc: AztecRPC) {}

  abstract getCompleteAddress(): CompleteAddress;

  abstract createTxExecutionRequest(execs: FunctionCall[]): Promise<TxExecutionRequest>;

  abstract createAuthWitness(message: Fr): Promise<AuthWitness>;

  registerAccount(privKey: GrumpkinPrivateKey, partialAddress: PartialAddress): Promise<void> {
    return this.rpc.registerAccount(privKey, partialAddress);
  }
  registerRecipient(account: CompleteAddress): Promise<void> {
    return this.rpc.registerRecipient(account);
  }
  getRegisteredAccounts(): Promise<CompleteAddress[]> {
    return this.rpc.getRegisteredAccounts();
  }
  getRegisteredAccount(address: AztecAddress): Promise<CompleteAddress | undefined> {
    return this.rpc.getRegisteredAccount(address);
  }
  getRecipients(): Promise<CompleteAddress[]> {
    return this.rpc.getRecipients();
  }
  getRecipient(address: AztecAddress): Promise<CompleteAddress | undefined> {
    return this.rpc.getRecipient(address);
  }
  addContracts(contracts: DeployedContract[]): Promise<void> {
    return this.rpc.addContracts(contracts);
  }
  getContracts(): Promise<AztecAddress[]> {
    return this.rpc.getContracts();
  }
  simulateTx(txRequest: TxExecutionRequest, simulatePublic: boolean): Promise<Tx> {
    return this.rpc.simulateTx(txRequest, simulatePublic);
  }
  sendTx(tx: Tx): Promise<TxHash> {
    return this.rpc.sendTx(tx);
  }
  getTx(txHash: TxHash): Promise<L2Tx | undefined> {
    return this.rpc.getTx(txHash);
  }
  getTxReceipt(txHash: TxHash): Promise<TxReceipt> {
    return this.rpc.getTxReceipt(txHash);
  }
  getPrivateStorageAt(owner: AztecAddress, contract: AztecAddress, storageSlot: Fr): Promise<NotePreimage[]> {
    return this.rpc.getPrivateStorageAt(owner, contract, storageSlot);
  }
  getPublicStorageAt(contract: AztecAddress, storageSlot: Fr): Promise<any> {
    return this.rpc.getPublicStorageAt(contract, storageSlot);
  }
  getNoteNonces(contract: AztecAddress, storageSlot: Fr, preimage: NotePreimage, txHash: TxHash): Promise<Fr[]> {
    return this.rpc.getNoteNonces(contract, storageSlot, preimage, txHash);
  }
  viewTx(functionName: string, args: any[], to: AztecAddress, from?: AztecAddress | undefined): Promise<any> {
    return this.rpc.viewTx(functionName, args, to, from);
  }
  getExtendedContractData(contractAddress: AztecAddress): Promise<ExtendedContractData | undefined> {
    return this.rpc.getExtendedContractData(contractAddress);
  }
  getContractData(contractAddress: AztecAddress): Promise<ContractData | undefined> {
    return this.rpc.getContractData(contractAddress);
  }
  getUnencryptedLogs(from: number, limit: number): Promise<L2BlockL2Logs[]> {
    return this.rpc.getUnencryptedLogs(from, limit);
  }
  getBlockNumber(): Promise<number> {
    return this.rpc.getBlockNumber();
  }
  getNodeInfo(): Promise<NodeInfo> {
    return this.rpc.getNodeInfo();
  }
  isGlobalStateSynchronized() {
    return this.rpc.isGlobalStateSynchronized();
  }
  isAccountStateSynchronized(account: AztecAddress) {
    return this.rpc.isAccountStateSynchronized(account);
  }
  getSyncStatus(): Promise<SyncStatus> {
    return this.rpc.getSyncStatus();
  }
  addAuthWitness(authWitness: AuthWitness) {
    return this.rpc.addAuthWitness(authWitness);
  }
}
