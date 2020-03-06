// wasi-sdk doesn't support the functions below. This file is hooked into stdlib.h by the Dockerfile.

inline int chdir(const char*)
{
    return 0;
}

inline char* getcwd(char* buf, size_t)
{
    buf[0] = '/';
    buf[1] = 0;
    return buf;
}
