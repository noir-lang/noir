### noirc_dbg tests

The debugger uses `barretenberg_blackbox_solver` that is based on wasmer runtime. The tests in debug mode fail because the solver calculates wrong offset. To have valid results run tests in release mode:
- from crate root directory
```bash
cargo test --release
```
- from repo root directory
```bash
cargo test -p noirc_dbg --release
```
