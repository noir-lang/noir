import {
  Fr,
  PUBLIC_CALL_STACK_LENGTH,
  PublicCallData,
  PublicCallStackItem,
  PublicCircuitPublicInputs,
  PublicKernelInputsNoPreviousKernel,
  PublicKernelPublicInputs,
  TxRequest,
} from '@aztec/circuits.js';
import { ContractDataSource, PublicTx, Tx } from '@aztec/types';
import { MerkleTreeOperations } from '@aztec/world-state';

import times from 'lodash.times';
import { Proof, PublicProver } from '../prover/index.js';
import { PublicCircuitSimulator, PublicKernelCircuitSimulator } from '../simulator/index.js';
import { ProcessedTx, makeEmptyProcessedTx, makeProcessedTx } from './processed_tx.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { getCombinedHistoricTreeRoots } from './utils.js';

/**
 * Converts Txs lifted from the P2P module into ProcessedTx objects by executing
 * any public function calls in them. Txs with private calls only are unaffected.
 */
export class PublicProcessor {
  constructor(
    protected db: MerkleTreeOperations,
    protected publicCircuit: PublicCircuitSimulator,
    protected publicKernel: PublicKernelCircuitSimulator,
    protected publicProver: PublicProver,
    protected contractDataSource: ContractDataSource,

    private log = createDebugLogger('aztec:sequencer:public-processor'),
  ) {}

  /**
   * Run each tx through the public circuit and the public kernel circuit if needed.
   * @param txs - Txs to process.
   * @returns The list of processed txs with their circuit simulation outputs.
   */
  public async process(txs: Tx[]): Promise<[ProcessedTx[], Tx[]]> {
    const result: ProcessedTx[] = [];
    const failed: Tx[] = [];

    for (const tx of txs) {
      this.log(`Processing tx ${await tx.getTxHash()}`);
      try {
        result.push(await this.processTx(tx));
      } catch (err) {
        this.log(`Error processing tx ${await tx.getTxHash()}: ${err}`);
        failed.push(tx);
      }
    }
    return [result, failed];
  }

  /**
   * Makes an empty processed tx. Useful for padding a block to a power of two number of txs.
   * @returns A processed tx with empty data.
   */
  public async makeEmptyProcessedTx() {
    const historicTreeRoots = await getCombinedHistoricTreeRoots(this.db);
    return makeEmptyProcessedTx(historicTreeRoots);
  }

  protected async processTx(tx: Tx): Promise<ProcessedTx> {
    if (tx.isPublic()) {
      const [publicKernelOutput, publicKernelProof] = await this.processPublicTx(tx);
      return makeProcessedTx(tx, publicKernelOutput, publicKernelProof);
    } else if (tx.isPrivate()) {
      return makeProcessedTx(tx);
    } else {
      return this.makeEmptyProcessedTx();
    }
  }

  // TODO: This is just picking up the txRequest and executing one iteration of it. It disregards
  // any existing private execution information, and any subsequent calls.
  protected async processPublicTx(tx: PublicTx): Promise<[PublicKernelPublicInputs, Proof]> {
    const { txRequest } = tx.txRequest;

    // TODO: Determine how to calculate bytecode hash. Circuits just check it isn't zero for now.
    // See https://github.com/AztecProtocol/aztec3-packages/issues/378
    const bytecodeHash = new Fr(1n);

    const circuitOutput = await this.publicCircuit.publicCircuit(txRequest);
    const circuitProof = await this.publicProver.getPublicCircuitProof(circuitOutput);
    const publicCallData = this.getPublicCallData(txRequest, bytecodeHash, circuitOutput, circuitProof);

    const publicKernelInput = new PublicKernelInputsNoPreviousKernel(tx.txRequest, publicCallData);
    const publicKernelOutput = await this.publicKernel.publicKernelCircuitNoInput(publicKernelInput);
    const publicKernelProof = await this.publicProver.getPublicKernelCircuitProof(publicKernelOutput);

    return [publicKernelOutput, publicKernelProof];
  }

  protected getPublicCallData(
    txRequest: TxRequest,
    bytecodeHash: Fr,
    publicCircuitOutput: PublicCircuitPublicInputs,
    publicCircuitProof: Proof,
  ) {
    // The first call is built from the tx request directly with an empty stack
    const contractAddress = txRequest.to;
    const callStackItem = new PublicCallStackItem(contractAddress, txRequest.functionData, publicCircuitOutput);
    const publicCallStackPreimages: PublicCallStackItem[] = times(PUBLIC_CALL_STACK_LENGTH, PublicCallStackItem.empty);
    const portalContractAddress = publicCircuitOutput.callContext.portalContractAddress.toField();

    // set the msgSender for each call in the call stack
    for (let i = 0; i < publicCallStackPreimages.length; i++) {
      const isDelegateCall = publicCallStackPreimages[i].publicInputs.callContext.isDelegateCall;
      publicCallStackPreimages[i].publicInputs.callContext.msgSender = isDelegateCall
        ? callStackItem.publicInputs.callContext.msgSender
        : callStackItem.contractAddress;
    }

    return new PublicCallData(
      callStackItem,
      publicCallStackPreimages,
      publicCircuitProof,
      portalContractAddress,
      bytecodeHash,
    );
  }
}
