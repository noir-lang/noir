import {
  type AuthWitness,
  type EventMetadata,
  type EventType,
  type ExtendedNote,
  type GetUnencryptedLogsResponse,
  type IncomingNotesFilter,
  type L2Block,
  type LogFilter,
  type OutgoingNotesFilter,
  type PXE,
  type PXEInfo,
  type SimulatedTx,
  type SyncStatus,
  type Tx,
  type TxEffect,
  type TxExecutionRequest,
  type TxHash,
  type TxReceipt,
} from '@aztec/circuit-types';
import { type NoteProcessorStats } from '@aztec/circuit-types/stats';
import {
  type AztecAddress,
  type CompleteAddress,
  type Fq,
  type Fr,
  type PartialAddress,
  type Point,
} from '@aztec/circuits.js';
import { type ContractArtifact } from '@aztec/foundation/abi';
import { type ContractClassWithId, type ContractInstanceWithAddress } from '@aztec/types/contracts';
import { type NodeInfo } from '@aztec/types/interfaces';

import { type Wallet } from '../account/wallet.js';
import { type ExecutionRequestInit } from '../entrypoint/entrypoint.js';
import { type IntentAction, type IntentInnerHash } from '../utils/authwit.js';

/**
 * A base class for Wallet implementations
 */
export abstract class BaseWallet implements Wallet {
  constructor(protected readonly pxe: PXE) {}

  abstract getCompleteAddress(): CompleteAddress;

  abstract getChainId(): Fr;

  abstract getVersion(): Fr;

  abstract createTxExecutionRequest(exec: ExecutionRequestInit): Promise<TxExecutionRequest>;

  abstract createAuthWit(intent: Fr | Buffer | IntentInnerHash | IntentAction): Promise<AuthWitness>;

  abstract rotateNullifierKeys(newNskM: Fq): Promise<void>;

  getAddress() {
    return this.getCompleteAddress().address;
  }
  getContractInstance(address: AztecAddress): Promise<ContractInstanceWithAddress | undefined> {
    return this.pxe.getContractInstance(address);
  }
  getContractClass(id: Fr): Promise<ContractClassWithId | undefined> {
    return this.pxe.getContractClass(id);
  }
  getContractArtifact(id: Fr): Promise<ContractArtifact | undefined> {
    return this.pxe.getContractArtifact(id);
  }
  addCapsule(capsule: Fr[]): Promise<void> {
    return this.pxe.addCapsule(capsule);
  }
  registerAccount(secretKey: Fr, partialAddress: PartialAddress): Promise<CompleteAddress> {
    return this.pxe.registerAccount(secretKey, partialAddress);
  }
  rotateNskM(address: AztecAddress, secretKey: Fq) {
    return this.pxe.rotateNskM(address, secretKey);
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
  registerContract(contract: {
    /** Instance */ instance: ContractInstanceWithAddress;
    /** Associated artifact */ artifact?: ContractArtifact;
  }): Promise<void> {
    return this.pxe.registerContract(contract);
  }
  registerContractClass(artifact: ContractArtifact): Promise<void> {
    return this.pxe.registerContractClass(artifact);
  }
  getContracts(): Promise<AztecAddress[]> {
    return this.pxe.getContracts();
  }
  proveTx(txRequest: TxExecutionRequest, simulatePublic: boolean): Promise<Tx> {
    return this.pxe.proveTx(txRequest, simulatePublic);
  }
  simulateTx(txRequest: TxExecutionRequest, simulatePublic: boolean, msgSender?: AztecAddress): Promise<SimulatedTx> {
    return this.pxe.simulateTx(txRequest, simulatePublic, msgSender);
  }
  sendTx(tx: Tx): Promise<TxHash> {
    return this.pxe.sendTx(tx);
  }
  getTxEffect(txHash: TxHash): Promise<TxEffect | undefined> {
    return this.pxe.getTxEffect(txHash);
  }
  getTxReceipt(txHash: TxHash): Promise<TxReceipt> {
    return this.pxe.getTxReceipt(txHash);
  }
  getIncomingNotes(filter: IncomingNotesFilter): Promise<ExtendedNote[]> {
    return this.pxe.getIncomingNotes(filter);
  }
  getOutgoingNotes(filter: OutgoingNotesFilter): Promise<ExtendedNote[]> {
    return this.pxe.getOutgoingNotes(filter);
  }
  // TODO(#4956): Un-expose this
  getNoteNonces(note: ExtendedNote): Promise<Fr[]> {
    return this.pxe.getNoteNonces(note);
  }
  getPublicStorageAt(contract: AztecAddress, storageSlot: Fr): Promise<any> {
    return this.pxe.getPublicStorageAt(contract, storageSlot);
  }
  addNote(note: ExtendedNote): Promise<void> {
    return this.pxe.addNote(note);
  }
  addNullifiedNote(note: ExtendedNote): Promise<void> {
    return this.pxe.addNullifiedNote(note);
  }
  getBlock(number: number): Promise<L2Block | undefined> {
    return this.pxe.getBlock(number);
  }
  simulateUnconstrained(
    functionName: string,
    args: any[],
    to: AztecAddress,
    from?: AztecAddress | undefined,
  ): Promise<any> {
    return this.pxe.simulateUnconstrained(functionName, args, to, from);
  }
  getUnencryptedLogs(filter: LogFilter): Promise<GetUnencryptedLogsResponse> {
    return this.pxe.getUnencryptedLogs(filter);
  }
  getBlockNumber(): Promise<number> {
    return this.pxe.getBlockNumber();
  }
  getProvenBlockNumber(): Promise<number> {
    return this.pxe.getProvenBlockNumber();
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
  getSyncStats(): Promise<{ [key: string]: NoteProcessorStats }> {
    return this.pxe.getSyncStats();
  }
  addAuthWitness(authWitness: AuthWitness) {
    return this.pxe.addAuthWitness(authWitness);
  }
  getAuthWitness(messageHash: Fr) {
    return this.pxe.getAuthWitness(messageHash);
  }
  isContractClassPubliclyRegistered(id: Fr): Promise<boolean> {
    return this.pxe.isContractClassPubliclyRegistered(id);
  }
  isContractPubliclyDeployed(address: AztecAddress): Promise<boolean> {
    return this.pxe.isContractPubliclyDeployed(address);
  }
  isContractInitialized(address: AztecAddress): Promise<boolean> {
    return this.pxe.isContractInitialized(address);
  }
  getPXEInfo(): Promise<PXEInfo> {
    return this.pxe.getPXEInfo();
  }
  getEvents<T>(
    type: EventType,
    eventMetadata: EventMetadata<T>,
    from: number,
    limit: number,
    vpks: Point[] = [
      this.getCompleteAddress().publicKeys.masterIncomingViewingPublicKey,
      this.getCompleteAddress().publicKeys.masterOutgoingViewingPublicKey,
    ],
  ) {
    return this.pxe.getEvents(type, eventMetadata, from, limit, vpks);
  }
}
