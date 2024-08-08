#!/usr/bin/env bash
echo "label: \"AztecJS\"" > ./docs/reference/developer_references/aztecjs/_category_.yml
mv ./docs/reference/developer_references/aztecjs ./processed-docs/reference/developer_references/aztecjs
mv ./docs/reference/developer_references/smart_contract_reference/aztec-nr ./processed-docs/reference/developer_references/smart_contract_reference/aztec-nr
