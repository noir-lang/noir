import { CommitmentsDB, PublicContractsDB, PublicStateDB } from '../../index.js';

/** - */
export class HostStorage {
  /** - */
  public readonly stateDb: PublicStateDB;
  /** - */
  public readonly contractsDb: PublicContractsDB;

  /** - */
  public readonly commitmentsDb: CommitmentsDB;

  constructor(stateDb: PublicStateDB, contractsDb: PublicContractsDB, commitmentsDb: CommitmentsDB) {
    this.stateDb = stateDb;
    this.contractsDb = contractsDb;
    this.commitmentsDb = commitmentsDb;
  }
}
