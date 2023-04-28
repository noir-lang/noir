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
import { ContractDataSource, PublicTx, Tx } from '@aztec/types';
import { MerkleTreeId, MerkleTreeOperations } from '@aztec/world-state';
import { pedersenGetHash } from '@aztec/barretenberg.js/crypto';
import { createDebugLogger } from '@aztec/foundation';
import times from 'lodash.times';
import { Proof, PublicProver } from '../prover/index.js';
import { PublicCircuitSimulator, PublicKernelCircuitSimulator } from '../simulator/index.js';
import { ProcessedTx, makeEmptyProcessedTx, makeProcessedTx } from './processed_tx.js';

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
   * @param txs - txs to process
   * @returns the list of processed txs with their circuit simulation outputs.
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

  protected async processTx(tx: Tx): Promise<ProcessedTx> {
    let processedTx: ProcessedTx;
    if (tx.isPublic()) {
      const [publicKernelOutput, publicKernelProof] = await this.processPublicTx(tx);
      processedTx = await makeProcessedTx(tx, publicKernelOutput, publicKernelProof);
    } else if (tx.isPrivate()) {
      processedTx = await makeProcessedTx(tx);
    } else {
      processedTx = await makeEmptyProcessedTx();
    }

    // TODO: this should be part of makeEmptyProcessedTx
    // set historic roots to in kernel tx to empty merkle tree root (instead of 0)
    if (processedTx.data.constants.historicTreeRoots.privateHistoricTreeRoots.privateDataTreeRoot.isZero()) {
      processedTx.data.constants.historicTreeRoots.privateHistoricTreeRoots.privateDataTreeRoot = Fr.fromBuffer(
        (await this.db.getTreeInfo(MerkleTreeId.PRIVATE_DATA_TREE)).root,
      );
    }
    if (processedTx.data.constants.historicTreeRoots.privateHistoricTreeRoots.contractTreeRoot.isZero()) {
      processedTx.data.constants.historicTreeRoots.privateHistoricTreeRoots.contractTreeRoot = Fr.fromBuffer(
        (await this.db.getTreeInfo(MerkleTreeId.CONTRACT_TREE)).root,
      );
    }

    return processedTx;
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

export class MockPublicProcessor extends PublicProcessor {
  protected processPublicTx(_tx: PublicTx): Promise<[PublicKernelPublicInputs, Proof]> {
    throw new Error('Public tx not supported by mock public processor');
  }
}
