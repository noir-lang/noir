// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

import {Test} from "forge-std/Test.sol";

contract DecoderBase is Test {
  struct AppendOnlyTreeSnapshot {
    uint32 nextAvailableLeafIndex;
    bytes32 root;
  }

  // When I had data and messages as one combined struct it failed, but I can have this top-layer and it works :shrug:
  // Note: Members of the struct (and substructs) have to be in ALPHABETICAL order!
  struct Full {
    Data block;
    Messages messages;
    Populate populate;
  }

  struct Populate {
    bytes32[] l1ToL2Content;
    bytes32 recipient;
    address sender;
  }

  struct Messages {
    bytes32[] l1ToL2Messages;
    bytes32[] l2ToL1Messages;
  }

  struct Data {
    bytes32 archive;
    bytes body;
    bytes32 calldataHash;
    DecodedHeader decodedHeader;
    bytes header;
    bytes32 l1ToL2MessagesHash;
    bytes32 publicInputsHash;
  }

  struct DecodedHeader {
    bytes32 bodyHash;
    GlobalVariables globalVariables;
    AppendOnlyTreeSnapshot lastArchive;
    StateReference stateReference;
  }

  struct GlobalVariables {
    uint256 blockNumber;
    uint256 chainId;
    address coinbase;
    bytes32 feeRecipient;
    uint256 timestamp;
    uint256 version;
  }

  struct StateReference {
    AppendOnlyTreeSnapshot l1ToL2MessageTree;
    PartialStateReference partialStateReference;
  }

  struct PartialStateReference {
    AppendOnlyTreeSnapshot contractTree;
    AppendOnlyTreeSnapshot noteHashTree;
    AppendOnlyTreeSnapshot nullifierTree;
    AppendOnlyTreeSnapshot publicDataTree;
  }

  function load(string memory name) public view returns (Full memory) {
    string memory root = vm.projectRoot();
    string memory path = string.concat(root, "/test/fixtures/", name, ".json");
    string memory json = vm.readFile(path);
    bytes memory json_bytes = vm.parseJson(json);
    Full memory full = abi.decode(json_bytes, (Full));
    return full;
  }
}
