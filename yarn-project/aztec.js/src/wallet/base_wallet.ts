import {
  AuthWitness,
  ExtendedNote,
  FunctionCall,
  GetUnencryptedLogsResponse,
  L2Block,
  LogFilter,
  NoteFilter,
  PXE,
  SyncStatus,
  Tx,
  TxEffect,
  TxExecutionRequest,
  TxHash,
  TxReceipt,
} from '@aztec/circuit-types';
import { AztecAddress, CompleteAddress, Fr, GrumpkinPrivateKey, PartialAddress } from '@aztec/circuits.js';
import { ContractArtifact } from '@aztec/foundation/abi';
import { ContractClassWithId, ContractInstanceWithAddress } from '@aztec/types/contracts';
import { NodeInfo } from '@aztec/types/interfaces';

import { Wallet } from '../account/wallet.js';
import { ContractFunctionInteraction } from '../contract/contract_function_interaction.js';
import { FeeOptions } from '../entrypoint/entrypoint.js';

/**
 * A base class for Wallet implementations
 */
export abstract class BaseWallet implements Wallet {
  constructor(protected readonly pxe: PXE) {}

  abstract getCompleteAddress(): CompleteAddress;

  abstract getChainId(): Fr;

  abstract getVersion(): Fr;

  abstract createTxExecutionRequest(execs: FunctionCall[], fee?: FeeOptions): Promise<TxExecutionRequest>;

  abstract createAuthWit(
    messageHashOrIntent:
      | Fr
      | Buffer
      | {
          /** The caller to approve  */
          caller: AztecAddress;
          /** The action to approve */
          action: ContractFunctionInteraction | FunctionCall;
          /** The chain id to approve */
          chainId?: Fr;
          /** The version to approve  */
          version?: Fr;
        },
  ): Promise<AuthWitness>;

  getAddress() {
    return this.getCompleteAddress().address;
  }
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
  simulateTx(txRequest: TxExecutionRequest, simulatePublic: boolean): Promise<Tx> {
    return this.pxe.simulateTx(txRequest, simulatePublic);
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
  getNotes(filter: NoteFilter): Promise<ExtendedNote[]> {
    return this.pxe.getNotes(filter);
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
  getBlock(number: number): Promise<L2Block | undefined> {
    return this.pxe.getBlock(number);
  }
  viewTx(functionName: string, args: any[], to: AztecAddress, from?: AztecAddress | undefined): Promise<any> {
    return this.pxe.viewTx(functionName, args, to, from);
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
  getAuthWitness(messageHash: Fr) {
    return this.pxe.getAuthWitness(messageHash);
  }
  isContractClassPubliclyRegistered(id: Fr): Promise<boolean> {
    return this.pxe.isContractClassPubliclyRegistered(id);
  }
  isContractPubliclyDeployed(address: AztecAddress): Promise<boolean> {
    return this.pxe.isContractPubliclyDeployed(address);
  }
}
