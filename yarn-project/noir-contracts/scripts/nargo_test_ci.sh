
#!/bin/bash

# Runs tests scripts for all contracts, then for all libraries.

./scripts/nargo_test.sh CONTRACT $(./scripts/get_all_contracts.sh)
./scripts/nargo_test.sh LIB $(./scripts/get_all_libraries.sh)