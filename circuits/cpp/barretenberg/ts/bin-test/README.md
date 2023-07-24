# bb.js Binary Test

This test runs a sequence of commands that tests a Noir double recursion circuit verifies (src/main.nr).
The circuit has already been compiled and its witness generated in nargo via `nargo compile && nargo execute witness`.

To test:

```
./bin-test.sh
```
