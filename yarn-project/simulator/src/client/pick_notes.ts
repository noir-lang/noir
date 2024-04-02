import { Comparator, type Note } from '@aztec/circuit-types';
import { Fr } from '@aztec/foundation/fields';

export interface PropertySelector {
  index: number;
  offset: number;
  length: number;
}

/**
 * Configuration for selecting values.
 */
export interface Select {
  /**
   * Selector of the field to select and match.
   */
  selector: PropertySelector;
  /**
   * Required value of the field.
   */
  value: Fr;
  /**
   * The comparator to use
   */
  comparator: Comparator;
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
   * Selector of the field to sort.
   */
  selector: PropertySelector;
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
 * Data needed from to perform sort.
 */
interface ContainsNote {
  /**
   * The note.
   */
  note: Note;
}

const selectPropertyFromSerializedNote = (noteData: Fr[], selector: PropertySelector): Fr => {
  const noteValueBuffer = noteData[selector.index].toBuffer();
  const noteValue = noteValueBuffer.subarray(selector.offset, selector.offset + selector.length);
  return Fr.fromBuffer(noteValue);
};

const selectNotes = <T extends ContainsNote>(noteDatas: T[], selects: Select[]): T[] =>
  noteDatas.filter(noteData =>
    selects.every(({ selector, value, comparator }) => {
      const noteValueFr = selectPropertyFromSerializedNote(noteData.note.items, selector);
      const comparatorSelector = {
        [Comparator.EQ]: () => noteValueFr.equals(value),
        [Comparator.NEQ]: () => !noteValueFr.equals(value),
        [Comparator.LT]: () => noteValueFr.lt(value),
        [Comparator.LTE]: () => noteValueFr.lt(value) || noteValueFr.equals(value),
        [Comparator.GT]: () => !noteValueFr.lt(value) && !noteValueFr.equals(value),
        [Comparator.GTE]: () => !noteValueFr.lt(value),
      };

      return comparatorSelector[comparator]();
    }),
  );

const sortNotes = (a: Fr[], b: Fr[], sorts: Sort[], level = 0): number => {
  if (sorts[level] === undefined) {
    return 0;
  }

  const { selector, order } = sorts[level];
  if (order === 0) {
    return 0;
  }

  const aValue = selectPropertyFromSerializedNote(a, selector);
  const bValue = selectPropertyFromSerializedNote(b, selector);

  const dir = order === 1 ? [-1, 1] : [1, -1];
  return aValue.toBigInt() === bValue.toBigInt()
    ? sortNotes(a, b, sorts, level + 1)
    : aValue.toBigInt() > bValue.toBigInt()
    ? dir[0]
    : dir[1];
};

/**
 * Pick from a note array a number of notes that meet the criteria.
 */
export function pickNotes<T extends ContainsNote>(
  noteDatas: T[],
  { selects = [], sorts = [], limit = 0, offset = 0 }: GetOptions,
): T[] {
  return selectNotes(noteDatas, selects)
    .sort((a, b) => sortNotes(a.note.items, b.note.items, sorts))
    .slice(offset, limit ? offset + limit : undefined);
}
