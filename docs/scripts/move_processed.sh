#!/usr/bin/env bash
echo "label: \"AztecJS\"" > ./docs/reference/aztecjs/_category_.yml
mv ./docs/reference/aztecjs ./processed-docs/reference/aztecjs
mv ./docs/reference/smart_contract_reference/aztec-nr ./processed-docs/reference/smart_contract_reference/aztec-nr
