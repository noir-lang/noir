/**
 * The key type for use with the kv-store
 */
export type Key = string | number | Array<string | number>;

/**
 * A range of keys to iterate over.
 */
export type Range<K extends Key = Key> = {
  /** The key of the first item to include */
  start?: K;
  /** The key of the last item to include */
  end?: K;
  /** Whether to iterate in reverse */
  reverse?: boolean;
  /** The maximum number of items to iterate over */
  limit?: number;
};
