import { createDebugOnlyLogger } from '../log/index.js';

/**
 * Dummy implementation of a necessary part of the wasi api:
 * https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md
 * We don't use these functions, but the environment expects them.
 * TODO find a way to update off of wasi 12.
 */
/* eslint-disable camelcase */
export const getEmptyWasiSdk = (debug = createDebugOnlyLogger('wasm:empty_wasi_sdk')) => ({
  /**
   * Retrieves the current time from the system clock.
   * This function is a dummy implementation of the WASI API's `clock_time_get` method,
   * which is expected by the environment but not used in this context.
   *
   * No input parameters or return values are required, as the purpose of this function
   * is solely to satisfy the environment expectations and provide debugging information.
   */
  clock_time_get() {
    debug('clock_time_get');
  },
  /**
   * Dummy implementation of WASI's environ_get function.
   * This function is used to obtain a snapshot of the current environment variables.
   * In this dummy implementation, no actual actions are performed, but the debug logger logs 'environ_get' when called.
   * Environment variables are not used in this context, so the real implementation is not required.
   *
   * @see https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#environ_get
   */
  environ_get() {
    debug('environ_get');
  },
  /**
   * Retrieves the environment variable sizes from the WebAssembly environment.
   * This function is part of the WASI API and provides a dummy implementation to fulfill the expected APIs.
   * It does not have any actual functionality, but serves as a placeholder in the environment.
   */
  environ_sizes_get() {
    debug('environ_sizes_get');
  },
  /**
   * Closes a file descriptor, releasing any resources associated with it.
   * This function does not perform any actual closing operation, but exists to
   * satisfy the requirements of the WebAssembly System Interface (WASI) API,
   * which expects certain functions to be present for compatibility purposes.
   *
   * @see https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md
   */
  fd_close() {
    debug('fd_close');
  },
  /**
   * A dummy implementation of the 'fd_read' function from the WASI API.
   * This function is required by the environment, but not used in this context.
   * It would normally read data from a file descriptor into an array buffer,
   * but here it simply logs the invocation for debugging purposes.
   */
  fd_read() {
    debug('fd_read');
  },
  /**
   * Handles the file descriptor write operation.
   * This dummy implementation of the WASI 'fd_write' function is part of the wasi API:
   * https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md
   * The environment expects this function, but it is not used in the current implementation.
   * It is used to write data from WebAssembly memory to a file descriptor.
   */
  fd_write() {
    debug('fd_write');
  },
  /**
   * Perform a file seek operation on the given file descriptor to change its current position.
   * The new position is calculated using the provided offset and whence values.
   * Throws an error if the file descriptor is invalid or the operation cannot be performed.
   *
   * @param fd - The file descriptor of the file to perform the seek operation on.
   * @param offset - The relative offset to apply, based on the whence value.
   * @param whence - The reference point from which the offset should be calculated. One of SEEK_SET (start), SEEK_CUR (current), or SEEK_END (end).
   * @returns The new position in the file after the seek operation has been performed.
   */
  fd_seek() {
    debug('fd_seek');
  },
  /**
   * This function is a dummy implementation of the 'fd_fdstat_get' function in the WebAssembly System Interface (WASI) API.
   * Although not actually used in this context, it is present due to the environment's expectation of its existence.
   * The 'fd_fdstat_get' function is typically responsible for obtaining file descriptor status information.
   */
  fd_fdstat_get() {
    debug('fd_fdstat_get');
  },
  /**
   * Sets the file descriptor flags for a given file descriptor.
   * This function is a dummy implementation of the WASI API function 'fd_fdstat_set_flags'.
   * It currently does not perform any operation but logs the function call with a debug instance.
   * This is provided since the environment expects this function to be present.
   */
  fd_fdstat_set_flags() {
    debug('fd_fdstat_set_flags');
  },
  /**
   * Handles the `fd_prestat_get` function call for the dummy WebAssembly System Interface (WASI) implementation.
   * This function is expected by the WASI environment, although it is not used in this implementation.
   * The `fd_prestat_get` function retrieves pre-opened file descriptor properties.
   *
   * @returns A constant integer value indicating successful completion of the function call.
   */
  fd_prestat_get() {
    debug('fd_prestat_get');
    return 8;
  },
  /**
   * Provides a dummy implementation for the `fd_prestat_dir_name` function, which is expected to be called by the WASI environment.
   * This function is intended to retrieve the pre-opened directory's path associated with the given file descriptor. However, since it's a dummy implementation,
   * it doesn't perform any actual operation and only logs the function call with the provided debug logger.
   *
   * @returns A constant number representing a dummy return value for the function call.
   */
  fd_prestat_dir_name() {
    debug('fd_prestat_dir_name');
    return 28;
  },
  /**
   * Handles the opening of a file path within the WASI environment.
   * This function is a dummy implementation required for compatibility with
   * the WebAssembly System Interface (WASI) API, but it does not perform any
   * actual file opening operation. It is mainly used for debugging purposes.
   */
  path_open() {
    debug('path_open');
  },
  /**
   * Retrieve file system information of the specified path.
   * This function retrieves statistics, such as size and permissions, associated with the file or directory
   * identified by the given path. In case of an error or non-existing path, appropriate debug logs will be generated.
   *
   * @returns An object containing file statistics like size, permissions, etc.
   */
  path_filestat_get() {
    debug('path_filestat_get');
  },
  /**
   * Terminate the process normally, performing the regular cleanup for terminating programs.
   * The input 'status' represents the exit code and is used to indicate success or failure
   * of the program execution. A zero value typically indicates successful execution,
   * while non-zero values are treated as errors by the operating system.
   *
   * @param status - The exit code representing the success or failure of the program execution.
   * @returns The exit status code.
   */
  proc_exit() {
    debug('proc_exit');
    return 52;
  },
  /**
   * Generates a random number and returns it.
   * This dummy implementation of 'random_get' method in the wasi API is expected by the environment.
   * In this case, the function always returns 1 to maintain consistency with the environment's expectations.
   *
   * @returns A random number. In this implementation, always returns 1.
   */
  random_get() {
    debug('random_get');
    return 1;
  },
});
