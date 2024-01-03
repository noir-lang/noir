#!/bin/bash
# Sadly, yarn-project expects ABI files in a format different to what is output from nargo.
# This provides a tool to quickly do the transform with jq.
# It's not currently used as the transform is done in TypeScript when we generate the contract wrapper.
# TODO: Why don't our contract classes just work with noir abis?
#
# Lowercase function_type value.
# Camel case function_type and is_internal.
# Discard first parameter (input) if function is not unconstrained.
# Hoist parameters out of abi.
# Hoist return_type out of abi, make an array of 1 element, or empty array if function is secret.
#
jq '
  .functions |= map(
    (.functionType = (.function_type | ascii_downcase)) |
    (.isInternal = .is_internal) |
    del(.function_type, .is_internal) |
    (.parameters = if .functionType == "unconstrained" then .abi.parameters else .abi.parameters[1:] end) |
    (.returnTypes = if .functionType == "secret" then [] else [ .abi.return_type.abi_type ] end) |
    del(.abi)
  )
' $1
