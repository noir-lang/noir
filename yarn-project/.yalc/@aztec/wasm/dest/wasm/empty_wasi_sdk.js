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
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiZW1wdHlfd2FzaV9zZGsuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvd2FzbS9lbXB0eV93YXNpX3Nkay50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLEVBQUUsaUJBQWlCLEVBQUUsTUFBTSxZQUFZLENBQUM7QUFFL0M7Ozs7O0dBS0c7QUFDSCw4QkFBOEI7QUFDOUIsd0NBQXdDO0FBQ3hDLE1BQU0sQ0FBQyxNQUFNLGVBQWUsR0FBRyxDQUFDLEtBQUssR0FBRyxpQkFBaUIsQ0FBQyxxQkFBcUIsQ0FBQyxFQUFFLEVBQUUsQ0FBQyxDQUFDO0lBQ3BGLGNBQWM7UUFDWixLQUFLLENBQUMsZ0JBQWdCLENBQUMsQ0FBQztJQUMxQixDQUFDO0lBQ0QsV0FBVztRQUNULEtBQUssQ0FBQyxhQUFhLENBQUMsQ0FBQztJQUN2QixDQUFDO0lBQ0QsaUJBQWlCO1FBQ2YsS0FBSyxDQUFDLG1CQUFtQixDQUFDLENBQUM7SUFDN0IsQ0FBQztJQUNELFFBQVE7UUFDTixLQUFLLENBQUMsVUFBVSxDQUFDLENBQUM7SUFDcEIsQ0FBQztJQUNELE9BQU87UUFDTCxLQUFLLENBQUMsU0FBUyxDQUFDLENBQUM7SUFDbkIsQ0FBQztJQUNELFFBQVE7UUFDTixLQUFLLENBQUMsVUFBVSxDQUFDLENBQUM7SUFDcEIsQ0FBQztJQUNELE9BQU87UUFDTCxLQUFLLENBQUMsU0FBUyxDQUFDLENBQUM7SUFDbkIsQ0FBQztJQUNELGFBQWE7UUFDWCxLQUFLLENBQUMsZUFBZSxDQUFDLENBQUM7SUFDekIsQ0FBQztJQUNELG1CQUFtQjtRQUNqQixLQUFLLENBQUMscUJBQXFCLENBQUMsQ0FBQztJQUMvQixDQUFDO0lBQ0QsY0FBYztRQUNaLEtBQUssQ0FBQyxnQkFBZ0IsQ0FBQyxDQUFDO1FBQ3hCLE9BQU8sQ0FBQyxDQUFDO0lBQ1gsQ0FBQztJQUNELG1CQUFtQjtRQUNqQixLQUFLLENBQUMscUJBQXFCLENBQUMsQ0FBQztRQUM3QixPQUFPLEVBQUUsQ0FBQztJQUNaLENBQUM7SUFDRCxTQUFTO1FBQ1AsS0FBSyxDQUFDLFdBQVcsQ0FBQyxDQUFDO0lBQ3JCLENBQUM7SUFDRCxpQkFBaUI7UUFDZixLQUFLLENBQUMsbUJBQW1CLENBQUMsQ0FBQztJQUM3QixDQUFDO0lBQ0QsU0FBUztRQUNQLEtBQUssQ0FBQyxXQUFXLENBQUMsQ0FBQztRQUNuQixPQUFPLEVBQUUsQ0FBQztJQUNaLENBQUM7SUFDRCxVQUFVO1FBQ1IsS0FBSyxDQUFDLFlBQVksQ0FBQyxDQUFDO1FBQ3BCLE9BQU8sQ0FBQyxDQUFDO0lBQ1gsQ0FBQztDQUNGLENBQUMsQ0FBQyJ9
