// To be provided by the environment.
// For barretenberg.wasm, this is provided by the JavaScript environment.
// For anything other than barretenberg.wasm, this is provided in this module.
extern "C" void logstr(char const*);

#ifdef __wasm__
inline void logstr_err(char const* err)
{
    logstr(err);
}
#else
extern "C" void logstr_err(char const*);
#endif
