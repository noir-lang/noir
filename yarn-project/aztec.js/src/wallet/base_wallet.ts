import { AztecAddress, Fr, GrumpkinPrivateKey, PartialAddress } from '@aztec/circuits.js';
import {
  AuthWitness,
  ContractData,
  DeployedContract,
  ExtendedContractData,
  FunctionCall,
  L2BlockL2Logs,
  L2Tx,
  NodeInfo,
  NotePreimage,
  PXE,
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
  constructor(protected readonly pxe: PXE) {}

  abstract getCompleteAddress(): CompleteAddress;

  abstract createTxExecutionRequest(execs: FunctionCall[]): Promise<TxExecutionRequest>;

  abstract createAuthWitness(message: Fr): Promise<AuthWitness>;

  registerAccount(privKey: GrumpkinPrivateKey, partialAddress: PartialAddress): Promise<void> {
    return this.pxe.registerAccount(privKey, partialAddress);
  }
  registerRecipient(account: CompleteAddress): Promise<void> {
    return this.pxe.registerRecipient(account);
  }
  getRegisteredAccounts(): Promise<CompleteAddress[]> {
    return this.pxe.getRegisteredAccounts();
  }
  getRegisteredAccount(address: AztecAddress): Promise<CompleteAddress | undefined> {
    return this.pxe.getRegisteredAccount(address);
  }
  getRecipients(): Promise<CompleteAddress[]> {
    return this.pxe.getRecipients();
  }
  getRecipient(address: AztecAddress): Promise<CompleteAddress | undefined> {
    return this.pxe.getRecipient(address);
  }
  addContracts(contracts: DeployedContract[]): Promise<void> {
    return this.pxe.addContracts(contracts);
  }
  getContracts(): Promise<AztecAddress[]> {
    return this.pxe.getContracts();
  }
  simulateTx(txRequest: TxExecutionRequest, simulatePublic: boolean): Promise<Tx> {
    return this.pxe.simulateTx(txRequest, simulatePublic);
  }
  sendTx(tx: Tx): Promise<TxHash> {
    return this.pxe.sendTx(tx);
  }
  getTx(txHash: TxHash): Promise<L2Tx | undefined> {
    return this.pxe.getTx(txHash);
  }
  getTxReceipt(txHash: TxHash): Promise<TxReceipt> {
    return this.pxe.getTxReceipt(txHash);
  }
  getPrivateStorageAt(owner: AztecAddress, contract: AztecAddress, storageSlot: Fr): Promise<NotePreimage[]> {
    return this.pxe.getPrivateStorageAt(owner, contract, storageSlot);
  }
  getPublicStorageAt(contract: AztecAddress, storageSlot: Fr): Promise<any> {
    return this.pxe.getPublicStorageAt(contract, storageSlot);
  }
  addNote(
    account: AztecAddress,
    contract: AztecAddress,
    storageSlot: Fr,
    preimage: NotePreimage,
    txHash: TxHash,
    nonce?: Fr,
  ): Promise<void> {
    return this.pxe.addNote(account, contract, storageSlot, preimage, txHash, nonce);
  }
  getNoteNonces(contract: AztecAddress, storageSlot: Fr, preimage: NotePreimage, txHash: TxHash): Promise<Fr[]> {
    return this.pxe.getNoteNonces(contract, storageSlot, preimage, txHash);
  }
  viewTx(functionName: string, args: any[], to: AztecAddress, from?: AztecAddress | undefined): Promise<any> {
    return this.pxe.viewTx(functionName, args, to, from);
  }
  getExtendedContractData(contractAddress: AztecAddress): Promise<ExtendedContractData | undefined> {
    return this.pxe.getExtendedContractData(contractAddress);
  }
  getContractData(contractAddress: AztecAddress): Promise<ContractData | undefined> {
    return this.pxe.getContractData(contractAddress);
  }
  getUnencryptedLogs(from: number, limit: number): Promise<L2BlockL2Logs[]> {
    return this.pxe.getUnencryptedLogs(from, limit);
  }
  getBlockNumber(): Promise<number> {
    return this.pxe.getBlockNumber();
  }
  getNodeInfo(): Promise<NodeInfo> {
    return this.pxe.getNodeInfo();
  }
  isGlobalStateSynchronized() {
    return this.pxe.isGlobalStateSynchronized();
  }
  isAccountStateSynchronized(account: AztecAddress) {
    return this.pxe.isAccountStateSynchronized(account);
  }
  getSyncStatus(): Promise<SyncStatus> {
    return this.pxe.getSyncStatus();
  }
  addAuthWitness(authWitness: AuthWitness) {
    return this.pxe.addAuthWitness(authWitness);
  }
}
