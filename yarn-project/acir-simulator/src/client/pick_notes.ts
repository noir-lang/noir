import { Fr } from '@aztec/foundation/fields';

/**
 * The order to sort an array.
 */
export enum SortOrder {
  NADA = 0,
  DESC = 1,
  ASC = 2,
}

/**
 * Options for selecting items from the database.
 */
interface GetOptions {
  /**
   * An array of indices of the fields to sort.
   * Default: empty array.
   */
  sortBy?: number[];
  /**
   * The order of the corresponding index in sortBy. (1: DESC, 2: ASC, 0: Do nothing)
   * Default: empty array.
   */
  sortOrder?: SortOrder[];
  /**
   * The number of items to retrieve per query.
   * Default: 0. No limit.
   */
  limit?: number;
  /**
   * The starting index for pagination.
   * Default: 0.
   */
  offset?: number;
}

/**
 * Basic data needed from a note to perform sort.
 */
interface BasicNoteData {
  /**
   * Preimage of a note.
   */
  preimage: Fr[];
}

const sortNotes = (a: Fr[], b: Fr[], sortBy: number[], sortOrder: number[], level = 0): number => {
  const index = sortBy[level];
  if (sortBy[level] === undefined) return 0;

  const order = sortOrder[level] ?? 1; // Default: Descending.
  if (order === 0) return 0;

  const dir = order === 1 ? [-1, 1] : [1, -1];
  return a[index].value === b[index].value
    ? sortNotes(a, b, sortBy, sortOrder, level + 1)
    : a[index].value > b[index].value
    ? dir[0]
    : dir[1];
};

/**
 * Pick from a note array a number of notes that meet the criteria.
 */
export function pickNotes<T extends BasicNoteData>(
  notes: T[],
  { sortBy = [], sortOrder = [], limit = 0, offset = 0 }: GetOptions,
) {
  return notes
    .sort((a, b) => sortNotes(a.preimage, b.preimage, sortBy, sortOrder))
    .slice(offset, limit ? offset + limit : undefined);
}
