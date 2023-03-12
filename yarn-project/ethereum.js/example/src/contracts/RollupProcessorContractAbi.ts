import { ContractAbi } from '@aztec/ethereum.js/contract';
export default new ContractAbi([
  {
    "inputs": [
      {
        "internalType": "uint256",
        "name": "_escapeBlockLowerBound",
        "type": "uint256"
      },
      {
        "internalType": "uint256",
        "name": "_escapeBlockUpperBound",
        "type": "uint256"
      }
    ],
    "stateMutability": "nonpayable",
    "type": "constructor"
  },
  {
    "inputs": [],
    "name": "ARRAY_OVERFLOW",
    "type": "error"
  },
  {
    "inputs": [
      {
        "internalType": "uint256",
        "name": "inputAssetId",
        "type": "uint256"
      }
    ],
    "name": "BRIDGE_WITH_IDENTICAL_INPUT_ASSETS",
    "type": "error"
  },
  {
    "inputs": [
      {
        "internalType": "uint256",
        "name": "outputAssetId",
        "type": "uint256"
      }
    ],
    "name": "BRIDGE_WITH_IDENTICAL_OUTPUT_ASSETS",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "DAILY_CAP_SURPASSED",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "DEPOSIT_TOKENS_WRONG_PAYMENT_TYPE",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "ENCODING_BYTE_INVALID",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "INCONSISTENT_BRIDGE_CALL_DATA",
    "type": "error"
  },
  {
    "inputs": [
      {
        "internalType": "uint256",
        "name": "providedIndex",
        "type": "uint256"
      },
      {
        "internalType": "uint256",
        "name": "expectedIndex",
        "type": "uint256"
      }
    ],
    "name": "INCORRECT_DATA_START_INDEX",
    "type": "error"
  },
  {
    "inputs": [
      {
        "internalType": "bytes32",
        "name": "providedDefiInteractionHash",
        "type": "bytes32"
      },
      {
        "internalType": "bytes32",
        "name": "expectedDefiInteractionHash",
        "type": "bytes32"
      }
    ],
    "name": "INCORRECT_PREVIOUS_DEFI_INTERACTION_HASH",
    "type": "error"
  },
  {
    "inputs": [
      {
        "internalType": "bytes32",
        "name": "oldStateHash",
        "type": "bytes32"
      },
      {
        "internalType": "bytes32",
        "name": "newStateHash",
        "type": "bytes32"
      }
    ],
    "name": "INCORRECT_STATE_HASH",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "INSUFFICIENT_DEPOSIT",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "INSUFFICIENT_ETH_PAYMENT",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "INSUFFICIENT_TOKEN_APPROVAL",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "INVALID_ADDRESS_NO_CODE",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "INVALID_ASSET_ADDRESS",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "INVALID_ASSET_GAS",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "INVALID_ASSET_ID",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "INVALID_BRIDGE_ADDRESS",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "INVALID_BRIDGE_CALL_DATA",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "INVALID_BRIDGE_GAS",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "INVALID_ESCAPE_BOUNDS",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "INVALID_PROVIDER",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "INVALID_ROLLUP_TOPOLOGY",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "INVALID_SIGNATURE",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "LOCKED_NO_REENTER",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "MSG_VALUE_WRONG_AMOUNT",
    "type": "error"
  },
  {
    "inputs": [
      {
        "internalType": "uint256",
        "name": "outputValue",
        "type": "uint256"
      }
    ],
    "name": "NONZERO_OUTPUT_VALUE_ON_NOT_USED_ASSET",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "NOT_PAUSED",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "PAUSED",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "PENDING_CAP_SURPASSED",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "PROOF_VERIFICATION_FAILED",
    "type": "error"
  },
  {
    "inputs": [
      {
        "internalType": "uint256",
        "name": "",
        "type": "uint256"
      },
      {
        "internalType": "uint256",
        "name": "",
        "type": "uint256"
      }
    ],
    "name": "PUBLIC_INPUTS_HASH_VERIFICATION_FAILED",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "SAFE_CAST_OVERFLOW",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "SIGNATURE_ADDRESS_IS_ZERO",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "SIGNATURE_RECOVERY_FAILED",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "THIRD_PARTY_CONTRACTS_FLAG_NOT_SET",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "WITHDRAW_TO_ZERO_ADDRESS",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "ZERO_TOTAL_INPUT_VALUE",
    "type": "error"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": false,
        "internalType": "bool",
        "name": "allowed",
        "type": "bool"
      }
    ],
    "name": "AllowThirdPartyContractsUpdated",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "uint256",
        "name": "assetId",
        "type": "uint256"
      },
      {
        "indexed": true,
        "internalType": "address",
        "name": "assetAddress",
        "type": "address"
      },
      {
        "indexed": false,
        "internalType": "uint256",
        "name": "assetGasLimit",
        "type": "uint256"
      }
    ],
    "name": "AssetAdded",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": false,
        "internalType": "uint256",
        "name": "assetId",
        "type": "uint256"
      },
      {
        "indexed": false,
        "internalType": "uint256",
        "name": "pendingCap",
        "type": "uint256"
      },
      {
        "indexed": false,
        "internalType": "uint256",
        "name": "dailyCap",
        "type": "uint256"
      }
    ],
    "name": "AssetCapUpdated",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "uint256",
        "name": "encodedBridgeCallData",
        "type": "uint256"
      },
      {
        "indexed": true,
        "internalType": "uint256",
        "name": "nonce",
        "type": "uint256"
      },
      {
        "indexed": false,
        "internalType": "uint256",
        "name": "totalInputValue",
        "type": "uint256"
      }
    ],
    "name": "AsyncDefiBridgeProcessed",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "uint256",
        "name": "bridgeAddressId",
        "type": "uint256"
      },
      {
        "indexed": true,
        "internalType": "address",
        "name": "bridgeAddress",
        "type": "address"
      },
      {
        "indexed": false,
        "internalType": "uint256",
        "name": "bridgeGasLimit",
        "type": "uint256"
      }
    ],
    "name": "BridgeAdded",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": false,
        "internalType": "bool",
        "name": "isCapped",
        "type": "bool"
      }
    ],
    "name": "CappedUpdated",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "uint256",
        "name": "encodedBridgeCallData",
        "type": "uint256"
      },
      {
        "indexed": true,
        "internalType": "uint256",
        "name": "nonce",
        "type": "uint256"
      },
      {
        "indexed": false,
        "internalType": "uint256",
        "name": "totalInputValue",
        "type": "uint256"
      },
      {
        "indexed": false,
        "internalType": "uint256",
        "name": "totalOutputValueA",
        "type": "uint256"
      },
      {
        "indexed": false,
        "internalType": "uint256",
        "name": "totalOutputValueB",
        "type": "uint256"
      },
      {
        "indexed": false,
        "internalType": "bool",
        "name": "result",
        "type": "bool"
      },
      {
        "indexed": false,
        "internalType": "bytes",
        "name": "errorReason",
        "type": "bytes"
      }
    ],
    "name": "DefiBridgeProcessed",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": false,
        "internalType": "address",
        "name": "defiBridgeProxy",
        "type": "address"
      }
    ],
    "name": "DefiBridgeProxyUpdated",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": false,
        "internalType": "uint32",
        "name": "delay",
        "type": "uint32"
      }
    ],
    "name": "DelayBeforeEscapeHatchUpdated",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "uint256",
        "name": "assetId",
        "type": "uint256"
      },
      {
        "indexed": true,
        "internalType": "address",
        "name": "depositorAddress",
        "type": "address"
      },
      {
        "indexed": false,
        "internalType": "uint256",
        "name": "depositValue",
        "type": "uint256"
      }
    ],
    "name": "Deposit",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": false,
        "internalType": "uint8",
        "name": "version",
        "type": "uint8"
      }
    ],
    "name": "Initialized",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "uint256",
        "name": "rollupId",
        "type": "uint256"
      },
      {
        "indexed": false,
        "internalType": "uint256",
        "name": "chunk",
        "type": "uint256"
      },
      {
        "indexed": false,
        "internalType": "uint256",
        "name": "totalChunks",
        "type": "uint256"
      },
      {
        "indexed": false,
        "internalType": "address",
        "name": "sender",
        "type": "address"
      }
    ],
    "name": "OffchainData",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": false,
        "internalType": "address",
        "name": "account",
        "type": "address"
      }
    ],
    "name": "Paused",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "bytes32",
        "name": "role",
        "type": "bytes32"
      },
      {
        "indexed": true,
        "internalType": "bytes32",
        "name": "previousAdminRole",
        "type": "bytes32"
      },
      {
        "indexed": true,
        "internalType": "bytes32",
        "name": "newAdminRole",
        "type": "bytes32"
      }
    ],
    "name": "RoleAdminChanged",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "bytes32",
        "name": "role",
        "type": "bytes32"
      },
      {
        "indexed": true,
        "internalType": "address",
        "name": "account",
        "type": "address"
      },
      {
        "indexed": true,
        "internalType": "address",
        "name": "sender",
        "type": "address"
      }
    ],
    "name": "RoleGranted",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "bytes32",
        "name": "role",
        "type": "bytes32"
      },
      {
        "indexed": true,
        "internalType": "address",
        "name": "account",
        "type": "address"
      },
      {
        "indexed": true,
        "internalType": "address",
        "name": "sender",
        "type": "address"
      }
    ],
    "name": "RoleRevoked",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "uint256",
        "name": "rollupId",
        "type": "uint256"
      },
      {
        "indexed": false,
        "internalType": "bytes32[]",
        "name": "nextExpectedDefiHashes",
        "type": "bytes32[]"
      },
      {
        "indexed": false,
        "internalType": "address",
        "name": "sender",
        "type": "address"
      }
    ],
    "name": "RollupProcessed",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "address",
        "name": "providerAddress",
        "type": "address"
      },
      {
        "indexed": false,
        "internalType": "bool",
        "name": "valid",
        "type": "bool"
      }
    ],
    "name": "RollupProviderUpdated",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": false,
        "internalType": "address",
        "name": "account",
        "type": "address"
      }
    ],
    "name": "Unpaused",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "address",
        "name": "verifierAddress",
        "type": "address"
      }
    ],
    "name": "VerifierUpdated",
    "type": "event"
  },
  {
    "inputs": [],
    "name": "DEFAULT_ADMIN_ROLE",
    "outputs": [
      {
        "internalType": "bytes32",
        "name": "",
        "type": "bytes32"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "EMERGENCY_ROLE",
    "outputs": [
      {
        "internalType": "bytes32",
        "name": "",
        "type": "bytes32"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "LISTER_ROLE",
    "outputs": [
      {
        "internalType": "bytes32",
        "name": "",
        "type": "bytes32"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "OWNER_ROLE",
    "outputs": [
      {
        "internalType": "bytes32",
        "name": "",
        "type": "bytes32"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "RESUME_ROLE",
    "outputs": [
      {
        "internalType": "bytes32",
        "name": "",
        "type": "bytes32"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "allowThirdPartyContracts",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes32",
        "name": "_proofHash",
        "type": "bytes32"
      }
    ],
    "name": "approveProof",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "uint256",
        "name": "",
        "type": "uint256"
      }
    ],
    "name": "assetGasLimits",
    "outputs": [
      {
        "internalType": "uint256",
        "name": "",
        "type": "uint256"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "uint256",
        "name": "",
        "type": "uint256"
      }
    ],
    "name": "asyncDefiInteractionHashes",
    "outputs": [
      {
        "internalType": "bytes32",
        "name": "",
        "type": "bytes32"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "uint256",
        "name": "",
        "type": "uint256"
      }
    ],
    "name": "bridgeGasLimits",
    "outputs": [
      {
        "internalType": "uint256",
        "name": "",
        "type": "uint256"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "uint256",
        "name": "",
        "type": "uint256"
      }
    ],
    "name": "caps",
    "outputs": [
      {
        "internalType": "uint128",
        "name": "available",
        "type": "uint128"
      },
      {
        "internalType": "uint32",
        "name": "lastUpdatedTimestamp",
        "type": "uint32"
      },
      {
        "internalType": "uint32",
        "name": "pendingCap",
        "type": "uint32"
      },
      {
        "internalType": "uint32",
        "name": "dailyCap",
        "type": "uint32"
      },
      {
        "internalType": "uint8",
        "name": "precision",
        "type": "uint8"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "defiBridgeProxy",
    "outputs": [
      {
        "internalType": "address",
        "name": "",
        "type": "address"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "uint256",
        "name": "",
        "type": "uint256"
      }
    ],
    "name": "defiInteractionHashes",
    "outputs": [
      {
        "internalType": "bytes32",
        "name": "",
        "type": "bytes32"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "delayBeforeEscapeHatch",
    "outputs": [
      {
        "internalType": "uint32",
        "name": "",
        "type": "uint32"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "uint256",
        "name": "_assetId",
        "type": "uint256"
      },
      {
        "internalType": "uint256",
        "name": "_amount",
        "type": "uint256"
      },
      {
        "internalType": "address",
        "name": "_owner",
        "type": "address"
      },
      {
        "internalType": "bytes32",
        "name": "_proofHash",
        "type": "bytes32"
      }
    ],
    "name": "depositPendingFunds",
    "outputs": [],
    "stateMutability": "payable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "address",
        "name": "",
        "type": "address"
      },
      {
        "internalType": "bytes32",
        "name": "",
        "type": "bytes32"
      }
    ],
    "name": "depositProofApprovals",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "escapeBlockLowerBound",
    "outputs": [
      {
        "internalType": "uint256",
        "name": "",
        "type": "uint256"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "escapeBlockUpperBound",
    "outputs": [
      {
        "internalType": "uint256",
        "name": "",
        "type": "uint256"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "uint256",
        "name": "",
        "type": "uint256"
      }
    ],
    "name": "ethPayments",
    "outputs": [
      {
        "internalType": "uint256",
        "name": "",
        "type": "uint256"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "getAsyncDefiInteractionHashesLength",
    "outputs": [
      {
        "internalType": "uint256",
        "name": "",
        "type": "uint256"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "getCapped",
    "outputs": [
      {
        "internalType": "bool",
        "name": "capped",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "getDataSize",
    "outputs": [
      {
        "internalType": "uint256",
        "name": "dataSize",
        "type": "uint256"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "getDefiInteractionHashesLength",
    "outputs": [
      {
        "internalType": "uint256",
        "name": "",
        "type": "uint256"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "getEscapeHatchStatus",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      },
      {
        "internalType": "uint256",
        "name": "",
        "type": "uint256"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "getImplementationVersion",
    "outputs": [
      {
        "internalType": "uint8",
        "name": "version",
        "type": "uint8"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "getPendingDefiInteractionHashesLength",
    "outputs": [
      {
        "internalType": "uint256",
        "name": "",
        "type": "uint256"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes32",
        "name": "role",
        "type": "bytes32"
      }
    ],
    "name": "getRoleAdmin",
    "outputs": [
      {
        "internalType": "bytes32",
        "name": "",
        "type": "bytes32"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "uint256",
        "name": "_assetId",
        "type": "uint256"
      }
    ],
    "name": "getSupportedAsset",
    "outputs": [
      {
        "internalType": "address",
        "name": "",
        "type": "address"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "getSupportedAssetsLength",
    "outputs": [
      {
        "internalType": "uint256",
        "name": "",
        "type": "uint256"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "uint256",
        "name": "_bridgeAddressId",
        "type": "uint256"
      }
    ],
    "name": "getSupportedBridge",
    "outputs": [
      {
        "internalType": "address",
        "name": "",
        "type": "address"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "getSupportedBridgesLength",
    "outputs": [
      {
        "internalType": "uint256",
        "name": "",
        "type": "uint256"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes32",
        "name": "role",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "account",
        "type": "address"
      }
    ],
    "name": "grantRole",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes32",
        "name": "role",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "account",
        "type": "address"
      }
    ],
    "name": "hasRole",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "initialize",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "lastRollupTimeStamp",
    "outputs": [
      {
        "internalType": "uint32",
        "name": "",
        "type": "uint32"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "uint256",
        "name": "_rollupId",
        "type": "uint256"
      },
      {
        "internalType": "uint256",
        "name": "_chunk",
        "type": "uint256"
      },
      {
        "internalType": "uint256",
        "name": "_totalChunks",
        "type": "uint256"
      },
      {
        "internalType": "bytes",
        "name": "",
        "type": "bytes"
      }
    ],
    "name": "offchainData",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "pause",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "paused",
    "outputs": [
      {
        "internalType": "bool",
        "name": "isPaused",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "uint256",
        "name": "",
        "type": "uint256"
      }
    ],
    "name": "pendingDefiInteractions",
    "outputs": [
      {
        "internalType": "uint256",
        "name": "encodedBridgeCallData",
        "type": "uint256"
      },
      {
        "internalType": "uint256",
        "name": "totalInputValue",
        "type": "uint256"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "prevDefiInteractionsHash",
    "outputs": [
      {
        "internalType": "bytes32",
        "name": "",
        "type": "bytes32"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "uint256",
        "name": "_interactionNonce",
        "type": "uint256"
      }
    ],
    "name": "processAsyncDefiInteraction",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes",
        "name": "",
        "type": "bytes"
      },
      {
        "internalType": "bytes",
        "name": "_signatures",
        "type": "bytes"
      }
    ],
    "name": "processRollup",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "uint256",
        "name": "_interactionNonce",
        "type": "uint256"
      }
    ],
    "name": "receiveEthFromBridge",
    "outputs": [],
    "stateMutability": "payable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes32",
        "name": "role",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "account",
        "type": "address"
      }
    ],
    "name": "renounceRole",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes32",
        "name": "role",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "account",
        "type": "address"
      }
    ],
    "name": "revokeRole",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "address",
        "name": "",
        "type": "address"
      }
    ],
    "name": "rollupProviders",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "rollupStateHash",
    "outputs": [
      {
        "internalType": "bytes32",
        "name": "",
        "type": "bytes32"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bool",
        "name": "_allowThirdPartyContracts",
        "type": "bool"
      }
    ],
    "name": "setAllowThirdPartyContracts",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "uint256",
        "name": "_assetId",
        "type": "uint256"
      },
      {
        "internalType": "uint32",
        "name": "_pendingCap",
        "type": "uint32"
      },
      {
        "internalType": "uint32",
        "name": "_dailyCap",
        "type": "uint32"
      },
      {
        "internalType": "uint8",
        "name": "_precision",
        "type": "uint8"
      }
    ],
    "name": "setAssetCap",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bool",
        "name": "_isCapped",
        "type": "bool"
      }
    ],
    "name": "setCapped",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "address",
        "name": "_defiBridgeProxy",
        "type": "address"
      }
    ],
    "name": "setDefiBridgeProxy",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "uint32",
        "name": "_delay",
        "type": "uint32"
      }
    ],
    "name": "setDelayBeforeEscapeHatch",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "address",
        "name": "_provider",
        "type": "address"
      },
      {
        "internalType": "bool",
        "name": "_valid",
        "type": "bool"
      }
    ],
    "name": "setRollupProvider",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "address",
        "name": "_token",
        "type": "address"
      },
      {
        "internalType": "uint256",
        "name": "_gasLimit",
        "type": "uint256"
      }
    ],
    "name": "setSupportedAsset",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "address",
        "name": "_bridge",
        "type": "address"
      },
      {
        "internalType": "uint256",
        "name": "_gasLimit",
        "type": "uint256"
      }
    ],
    "name": "setSupportedBridge",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "address",
        "name": "_verifier",
        "type": "address"
      }
    ],
    "name": "setVerifier",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes4",
        "name": "interfaceId",
        "type": "bytes4"
      }
    ],
    "name": "supportsInterface",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "unpause",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "uint256",
        "name": "",
        "type": "uint256"
      },
      {
        "internalType": "address",
        "name": "",
        "type": "address"
      }
    ],
    "name": "userPendingDeposits",
    "outputs": [
      {
        "internalType": "uint256",
        "name": "",
        "type": "uint256"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "verifier",
    "outputs": [
      {
        "internalType": "address",
        "name": "",
        "type": "address"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  }
]);