/**
 * @description
 * An options object, currently only used to specify the number of threads to use.
 */
export type BackendOptions = {
  /** @description Number of threads */
  threads: number;

  memory?: { maximum: number };

  /** @description The path to download CRS files */
  crsPath?: string;
};
