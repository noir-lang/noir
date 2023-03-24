/**
 * Dummy implementation of a necessary part of the wasi api:
 * https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md
 * We don't use these functions, but the environment expects them.
 * TODO find a way to update off of wasi 12.
 */
export declare const getEmptyWasiSdk: (
  debug?: import("@aztec/foundation").DebugLogger
) => {
  clock_time_get(): void;
  environ_get(): void;
  environ_sizes_get(): void;
  fd_close(): void;
  fd_read(): void;
  fd_write(): void;
  fd_seek(): void;
  fd_fdstat_get(): void;
  fd_fdstat_set_flags(): void;
  fd_prestat_get(): number;
  fd_prestat_dir_name(): number;
  path_open(): void;
  path_filestat_get(): void;
  proc_exit(): number;
  random_get(): number;
};
//# sourceMappingURL=empty_wasi_sdk.d.ts.map
