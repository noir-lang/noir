import { Fr } from '@aztec/foundation/fields';

/**
 * Configuration for selecting values.
 */
export interface Select {
  /**
   * Index of the field to select and match.
   */
  index: number;
  /**
   * Required value of the field.
   */
  value: Fr;
}

/**
 * The order to sort an array.
 */
export enum SortOrder {
  NADA = 0,
  DESC = 1,
  ASC = 2,
}

/**
 * Configuration for sorting values.
 */
export interface Sort {
  /**
   * Index of the field to sort.
   */
  index: number;
  /**
   * Order to sort the field.
   */
  order: SortOrder;
}

/**
 * Options for picking items from an array of BasicNoteData.
 */
interface GetOptions {
  /**
   * Configurations for selecting items.
   * Default: empty array.
   */
  selects?: Select[];
  /**
   * Configurations for sorting items.
   * Default: empty array.
   */
  sorts?: Sort[];
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

const selectNotes = <T extends BasicNoteData>(notes: T[], selects: Select[]): T[] =>
  notes.filter(note => selects.every(({ index, value }) => note.preimage[index]?.equals(value)));

const sortNotes = (a: Fr[], b: Fr[], sorts: Sort[], level = 0): number => {
  if (sorts[level] === undefined) return 0;

  const { index, order } = sorts[level];
  if (order === 0) return 0;

  const dir = order === 1 ? [-1, 1] : [1, -1];
  return a[index].value === b[index].value
    ? sortNotes(a, b, sorts, level + 1)
    : a[index].value > b[index].value
    ? dir[0]
    : dir[1];
};

/**
 * Pick from a note array a number of notes that meet the criteria.
 */
export function pickNotes<T extends BasicNoteData>(
  notes: T[],
  { selects = [], sorts = [], limit = 0, offset = 0 }: GetOptions,
): T[] {
  return selectNotes(notes, selects)
    .sort((a, b) => sortNotes(a.preimage, b.preimage, sorts))
    .slice(offset, limit ? offset + limit : undefined);
}
