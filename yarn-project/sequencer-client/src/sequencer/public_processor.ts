import { pedersenGetHash } from '@aztec/barretenberg.js/crypto';
import {
  CircuitsWasm,
  Fr,
  PUBLIC_CALL_STACK_LENGTH,
  PublicCallData,
  PublicCallStackItem,
  PublicCircuitPublicInputs,
  PublicKernelInputsNoPreviousKernel,
  PublicKernelPublicInputs,
  TxRequest,
} from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation';
import { ContractDataSource, PublicTx, Tx } from '@aztec/types';
import { MerkleTreeOperations } from '@aztec/world-state';
import times from 'lodash.times';
import { Proof, PublicProver } from '../prover/index.js';
import { PublicCircuitSimulator, PublicKernelCircuitSimulator } from '../simulator/index.js';
import { ProcessedTx, makeEmptyProcessedTx, makeProcessedTx } from './processed_tx.js';
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
    const contractAddress = txRequest.to;

    const fn = await this.contractDataSource.getPublicFunction(
      contractAddress,
      txRequest.functionData.functionSelector,
    );
    const functionSelector = txRequest.functionData.functionSelector;
    if (!fn) throw new Error(`Bytecode not found for ${functionSelector}@${contractAddress}`);
    const contractPublicData = await this.contractDataSource.getL2ContractPublicData(contractAddress);
    if (!contractPublicData) throw new Error(`Portal contract address not found for contract ${contractAddress}`);
    const { portalContractAddress } = contractPublicData.contractData;

    const circuitOutput = await this.publicCircuit.publicCircuit(txRequest, fn.bytecode, portalContractAddress);
    const circuitProof = await this.publicProver.getPublicCircuitProof(circuitOutput);
    const publicCallData = await this.getPublicCallData(txRequest, fn.bytecode, circuitOutput, circuitProof);

    const publicKernelInput = new PublicKernelInputsNoPreviousKernel(tx.txRequest, publicCallData);
    const publicKernelOutput = await this.publicKernel.publicKernelCircuitNoInput(publicKernelInput);
    const publicKernelProof = await this.publicProver.getPublicKernelCircuitProof(publicKernelOutput);

    return [publicKernelOutput, publicKernelProof];
  }

  protected async getPublicCallData(
    txRequest: TxRequest,
    functionBytecode: Buffer,
    publicCircuitOutput: PublicCircuitPublicInputs,
    publicCircuitProof: Proof,
  ) {
    // The first call is built from the tx request directly with an empty stack
    const contractAddress = txRequest.to;
    const callStackItem = new PublicCallStackItem(contractAddress, txRequest.functionData, publicCircuitOutput);
    const publicCallStackPreimages: PublicCallStackItem[] = times(PUBLIC_CALL_STACK_LENGTH, PublicCallStackItem.empty);

    // TODO: Determine how to calculate bytecode hash
    // See https://github.com/AztecProtocol/aztec3-packages/issues/378
    const bytecodeHash = Fr.fromBuffer(pedersenGetHash(await CircuitsWasm.get(), functionBytecode));
    const portalContractAddress = publicCircuitOutput.callContext.portalContractAddress.toField();

    return new PublicCallData(
      callStackItem,
      publicCallStackPreimages,
      publicCircuitProof,
      portalContractAddress,
      bytecodeHash,
    );
  }
}
