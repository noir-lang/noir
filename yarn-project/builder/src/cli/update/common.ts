/**
 * Tracks changes to dependencies
 */
export type DependencyChanges = {
  /** Which file was changed */
  file: string;
  /** changes done to the file */
  dependencies: Array<{
    /** Name of the dependency being changed */
    name: string;
    /** Previous version of the dependency */
    from: string;
    /** New version of the dependency (after the update) */
    to: string;
  }>;
};
