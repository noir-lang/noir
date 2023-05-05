import { PublicStateDB, PublicExecution, PublicExecutor, PublicContractsDB } from '@aztec/acir-simulator';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import { AztecAddress, EthAddress, Fr, PublicCircuitPublicInputs, TxRequest } from '@aztec/circuits.js';
import { MerkleTreeId } from '@aztec/types';
import { MerkleTreeOperations, computePublicDataTreeLeafIndex } from '@aztec/world-state';
import { PublicCircuitSimulator } from './index.js';
import { ContractDataSource } from '@aztec/types';

/**
 * Emulates the PublicCircuit simulator by executing ACIR as if it were Brillig opcodes.
 */
export class FakePublicCircuitSimulator implements PublicCircuitSimulator {
  private readonly stateDb: WorldStatePublicDB;
  private readonly contractsDb: ContractsDataSourcePublicDB;

  constructor(
    private readonly merkleTree: MerkleTreeOperations,
    private readonly contractDataSource: ContractDataSource,
  ) {
    this.stateDb = new WorldStatePublicDB(this.merkleTree);
    this.contractsDb = new ContractsDataSourcePublicDB(this.contractDataSource);
  }

  /**
   * Emulates a simulation of the public circuit for the given tx.
   * @param tx - Transaction request to execute.
   * @param functionBytecode - Bytecode (ACIR for now) of the function to run.
   * @param portalAddress - Corresponding portal address of the contract being run.
   * @returns The resulting PublicCircuitPublicInputs as if the circuit had been simulated.
   */
  public async publicCircuit(tx: TxRequest): Promise<PublicCircuitPublicInputs> {
    const publicDataTreeInfo = await this.merkleTree.getTreeInfo(MerkleTreeId.PUBLIC_DATA_TREE);
    const historicPublicDataTreeRoot = Fr.fromBuffer(publicDataTreeInfo.root);

    const executor = new PublicExecutor(this.stateDb, this.contractsDb);
    const execution = await executor.getPublicExecution(tx);
    const result = await executor.execute(execution);

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
 * Implements the PublicContractsDB using a ContractDataSource.
 */
class ContractsDataSourcePublicDB implements PublicContractsDB {
  constructor(private db: ContractDataSource) {}
  async getBytecode(address: AztecAddress, functionSelector: Buffer): Promise<Buffer | undefined> {
    return (await this.db.getPublicFunction(address, functionSelector))?.bytecode;
  }
  async getPortalContractAddress(address: AztecAddress): Promise<EthAddress | undefined> {
    return (await this.db.getL2ContractInfo(address))?.portalContractAddress;
  }
}

/**
 * Implements the PublicStateDB using a world-state database.
 */
class WorldStatePublicDB implements PublicStateDB {
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
