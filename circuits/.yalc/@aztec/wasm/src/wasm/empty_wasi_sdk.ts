import { createDebugLogger } from "@aztec/foundation";

/**
 * Dummy implementation of a necessary part of the wasi api:
 * https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md
 * We don't use these functions, but the environment expects them.
 * TODO find a way to update off of wasi 12.
 */
/* eslint-disable camelcase */
/* eslint-disable jsdoc/require-jsdoc */
export const getEmptyWasiSdk = (
  debug = createDebugLogger("wasm:empty_wasi_sdk")
) => ({
  clock_time_get() {
    debug("clock_time_get");
  },
  environ_get() {
    debug("environ_get");
  },
  environ_sizes_get() {
    debug("environ_sizes_get");
  },
  fd_close() {
    debug("fd_close");
  },
  fd_read() {
    debug("fd_read");
  },
  fd_write() {
    debug("fd_write");
  },
  fd_seek() {
    debug("fd_seek");
  },
  fd_fdstat_get() {
    debug("fd_fdstat_get");
  },
  fd_fdstat_set_flags() {
    debug("fd_fdstat_set_flags");
  },
  fd_prestat_get() {
    debug("fd_prestat_get");
    return 8;
  },
  fd_prestat_dir_name() {
    debug("fd_prestat_dir_name");
    return 28;
  },
  path_open() {
    debug("path_open");
  },
  path_filestat_get() {
    debug("path_filestat_get");
  },
  proc_exit() {
    debug("proc_exit");
    return 52;
  },
  random_get() {
    debug("random_get");
    return 1;
  },
});
