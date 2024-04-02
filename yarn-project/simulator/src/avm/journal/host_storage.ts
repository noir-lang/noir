import { type CommitmentsDB, type PublicContractsDB, type PublicStateDB } from '../../public/db.js';

/**
 * Host storage
 *
 * A wrapper around the node dbs
 */
export class HostStorage {
  constructor(
    public readonly publicStateDb: PublicStateDB,
    public readonly contractsDb: PublicContractsDB,
    public readonly commitmentsDb: CommitmentsDB,
  ) {}
}
