/**
 * Error thrown when a base journal is attempted to be merged.
 */
export class RootJournalCannotBeMerged extends Error {
  constructor() {
    super('Root journal cannot be merged');
  }
}
