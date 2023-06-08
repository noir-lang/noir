// If building WASM, we can stub out functions we know we don't need, to save the host
// environment from having to stub them itself.
#include <cstdint>
#include <cstdlib>
#include <barretenberg/common/log.hpp>

extern "C" {

int32_t __imported_wasi_snapshot_preview1_sched_yield()
{
    return 0;
}

int32_t __imported_wasi_snapshot_preview1_poll_oneoff(int32_t, int32_t, int32_t, int32_t)
{
    info("poll_oneoff not implemented.");
    abort();
}

// void __imported_wasi_snapshot_preview1_proc_exit(int32_t)
// {
//     info("proc_exit not implemented.");
//     abort();
// }

int32_t __imported_wasi_snapshot_preview1_fd_write(int32_t, int32_t, int32_t, int32_t)
{
    info("fd_write not implemented.");
    abort();
    return 0;
}

int32_t __imported_wasi_snapshot_preview1_fd_seek(int32_t, int64_t, int32_t, int32_t)
{
    info("fd_seek not implemented.");
    abort();
    return 0;
}

int32_t __imported_wasi_snapshot_preview1_fd_close(int32_t)
{
    info("fd_close not implemented.");
    abort();
    return 0;
}

int32_t __imported_wasi_snapshot_preview1_environ_get(int32_t, int32_t)
{
    info("environ_get not implemented.");
    abort();
    return 0;
}

int32_t __imported_wasi_snapshot_preview1_environ_sizes_get(int32_t, int32_t)
{
    // info("environ_sizes_get not implemented.");
    // abort();
    return 0;
}

// int32_t __imported_wasi_snapshot_preview1_clock_time_get(int32_t, int64_t, int32_t)
// {
//     info("clock_time_get not implemented.");
//     abort();
//     return 0;
// }

int32_t __imported_wasi_snapshot_preview1_fd_fdstat_get(int32_t, int32_t)
{
    info("fd_fdstat_get not implemented.");
    abort();
    return 0;
}

int32_t __imported_wasi_snapshot_preview1_fd_fdstat_set_flags(int32_t, int32_t)
{
    info("fd_fdstat_set_flags not implemented.");
    abort();
    return 0;
}

int32_t __imported_wasi_snapshot_preview1_fd_read(int32_t, int32_t, int32_t, int32_t)
{
    info("fd_read not implemented.");
    abort();
    return 0;
}

int32_t __imported_wasi_snapshot_preview1_path_open(
    int32_t, int32_t, int32_t, int32_t, int32_t, int64_t, int64_t, int32_t, int32_t)
{
    info("path_open not implemented.");
    abort();
    return 0;
}

int32_t __imported_wasi_snapshot_preview1_fd_prestat_get(int32_t, int32_t)
{
    // info("fd_prestat_get not implemented.");
    // abort();
    return 8;
}

int32_t __imported_wasi_snapshot_preview1_fd_prestat_dir_name(int32_t, int32_t, int32_t)
{
    info("fd_prestat_dir_name not implemented.");
    abort();
    return 28;
}

int32_t __imported_wasi_snapshot_preview1_path_filestat_get(int32_t, int32_t, int32_t, int32_t, int32_t)
{
    return 0;
}
}