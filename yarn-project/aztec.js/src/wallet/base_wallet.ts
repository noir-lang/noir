import {
  AuthWitness,
  ContractData,
  DeployedContract,
  ExtendedContractData,
  ExtendedNote,
  FunctionCall,
  GetUnencryptedLogsResponse,
  L2Block,
  L2Tx,
  LogFilter,
  NoteFilter,
  PXE,
  SyncStatus,
  Tx,
  TxExecutionRequest,
  TxHash,
  TxReceipt,
} from '@aztec/circuit-types';
import { AztecAddress, CompleteAddress, Fr, GrumpkinPrivateKey, PartialAddress } from '@aztec/circuits.js';
import { ContractClassWithId, ContractInstanceWithAddress } from '@aztec/types/contracts';
import { NodeInfo } from '@aztec/types/interfaces';

import { FeeOptions } from '../account/interface.js';
import { Wallet } from '../account/wallet.js';

/**
 * A base class for Wallet implementations
 */
export abstract class BaseWallet implements Wallet {
  constructor(protected readonly pxe: PXE) {}

  abstract getCompleteAddress(): CompleteAddress;

  abstract createTxExecutionRequest(execs: FunctionCall[], fee?: FeeOptions): Promise<TxExecutionRequest>;

  abstract createAuthWitness(message: Fr): Promise<AuthWitness>;

  getContractInstance(address: AztecAddress): Promise<ContractInstanceWithAddress | undefined> {
    return this.pxe.getContractInstance(address);
  }
  getContractClass(id: Fr): Promise<ContractClassWithId | undefined> {
    return this.pxe.getContractClass(id);
  }
  addCapsule(capsule: Fr[]): Promise<void> {
    return this.pxe.addCapsule(capsule);
  }
  registerAccount(privKey: GrumpkinPrivateKey, partialAddress: PartialAddress): Promise<CompleteAddress> {
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
  getNotes(filter: NoteFilter): Promise<ExtendedNote[]> {
    return this.pxe.getNotes(filter);
  }
  getPublicStorageAt(contract: AztecAddress, storageSlot: Fr): Promise<any> {
    return this.pxe.getPublicStorageAt(contract, storageSlot);
  }
  addNote(note: ExtendedNote): Promise<void> {
    return this.pxe.addNote(note);
  }
  getBlock(number: number): Promise<L2Block | undefined> {
    return this.pxe.getBlock(number);
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
  getUnencryptedLogs(filter: LogFilter): Promise<GetUnencryptedLogsResponse> {
    return this.pxe.getUnencryptedLogs(filter);
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
  isContractClassPubliclyRegistered(id: Fr): Promise<boolean> {
    return this.pxe.isContractClassPubliclyRegistered(id);
  }
}
