import { PublicDB, PublicExecution } from '@aztec/acir-simulator';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import { AztecAddress, EthAddress, Fr, PublicCircuitPublicInputs, TxRequest } from '@aztec/circuits.js';
import { MerkleTreeId, MerkleTreeOperations, computePublicDataTreeLeafIndex } from '@aztec/world-state';
import { PublicCircuitSimulator } from './index.js';

/**
 * Emulates the PublicCircuit simulator by executing ACIR as if it were Brillig opcodes.
 */
export class FakePublicCircuitSimulator implements PublicCircuitSimulator {
  private readonly db: WorldStatePublicDB;

  constructor(private readonly merkleTree: MerkleTreeOperations) {
    this.db = new WorldStatePublicDB(this.merkleTree);
  }

  /**
   * Emulates a simulation of the public circuit for the given tx.
   * @param tx - Transaction request to execute.
   * @param functionBytecode - Bytecode (ACIR for now) of the function to run.
   * @param portalAddress - Corresponding portal address of the contract being run.
   * @returns The resulting PublicCircuitPublicInputs as if the circuit had been simulated.
   */
  public async publicCircuit(
    tx: TxRequest,
    functionBytecode: Buffer,
    portalAddress: EthAddress,
  ): Promise<PublicCircuitPublicInputs> {
    const publicDataTreeInfo = await this.merkleTree.getTreeInfo(MerkleTreeId.PUBLIC_DATA_TREE);
    const historicPublicDataTreeRoot = Fr.fromBuffer(publicDataTreeInfo.root);

    const execution = PublicExecution.fromTransactionRequest(this.db, tx, functionBytecode, portalAddress);
    const result = await execution.run();
    return PublicCircuitPublicInputs.from({
      args: tx.args,
      callContext: execution.callContext,
      emittedEvents: [],
      newL2ToL1Msgs: [],
      proverAddress: AztecAddress.random(),
      publicCallStack: [],
      returnValues: result.returnValues,
      stateReads: result.stateReads,
      stateTransitions: result.stateTransitions,
      historicPublicDataTreeRoot,
    });
  }
}

/**
 * Implements the PublicDB using a world-state database.
 */
class WorldStatePublicDB implements PublicDB {
  constructor(private db: MerkleTreeOperations) {}

  /**
   * Reads a value from public storage, returning zero if none.
   * @param contract - Owner of the storage.
   * @param slot - Slot to read in the contract storage.
   * @returns The current value in the storage slot.
   */
  public async storageRead(contract: AztecAddress, slot: Fr): Promise<Fr> {
    const index = computePublicDataTreeLeafIndex(contract, slot, await BarretenbergWasm.get());
    const value = await this.db.getLeafValue(MerkleTreeId.PUBLIC_DATA_TREE, index);
    return value ? Fr.fromBuffer(value) : Fr.ZERO;
  }
}
