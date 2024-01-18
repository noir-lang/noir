import { CommitmentsDB, PublicContractsDB, PublicStateDB } from '../../index.js';

/**
 * Host storage
 *
 * A wrapper around the node dbs
 */
export class HostStorage {
  /** - */
  public readonly publicStateDb: PublicStateDB;
  /** - */
  public readonly contractsDb: PublicContractsDB;
  /** - */
  public readonly commitmentsDb: CommitmentsDB;

  constructor(publicStateDb: PublicStateDB, contractsDb: PublicContractsDB, commitmentsDb: CommitmentsDB) {
    this.publicStateDb = publicStateDb;
    this.contractsDb = contractsDb;
    this.commitmentsDb = commitmentsDb;
  }
}
