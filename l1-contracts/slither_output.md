Summary
 - [pess-unprotected-setter](#pess-unprotected-setter) (1 results) (High)
 - [uninitialized-local](#uninitialized-local) (2 results) (Medium)
 - [pess-dubious-typecast](#pess-dubious-typecast) (4 results) (Medium)
 - [missing-zero-check](#missing-zero-check) (2 results) (Low)
 - [reentrancy-events](#reentrancy-events) (2 results) (Low)
 - [timestamp](#timestamp) (1 results) (Low)
 - [pess-public-vs-external](#pess-public-vs-external) (5 results) (Low)
 - [assembly](#assembly) (1 results) (Informational)
 - [dead-code](#dead-code) (5 results) (Informational)
 - [solc-version](#solc-version) (1 results) (Informational)
 - [similar-names](#similar-names) (3 results) (Informational)
 - [constable-states](#constable-states) (1 results) (Optimization)
 - [pess-multiple-storage-read](#pess-multiple-storage-read) (6 results) (Optimization)
## pess-unprotected-setter
Impact: High
Confidence: Medium
 - [ ] ID-0
Function [Rollup.process(bytes,bytes32,bytes)](src/core/Rollup.sol#L58-L96) is a non-protected setter archive is written

src/core/Rollup.sol#L58-L96


## uninitialized-local
Impact: Medium
Confidence: Medium
 - [ ] ID-1
[HeaderLib.decode(bytes).header](src/core/libraries/HeaderLib.sol#L148) is a local variable never initialized

src/core/libraries/HeaderLib.sol#L148


 - [ ] ID-2
[TxsDecoder.decode(bytes).vars](src/core/libraries/decoders/TxsDecoder.sol#L78) is a local variable never initialized

src/core/libraries/decoders/TxsDecoder.sol#L78


## pess-dubious-typecast
Impact: Medium
Confidence: High
 - [ ] ID-3
Dubious typecast in [Hash.sha256ToField(bytes)](src/core/libraries/Hash.sol#L42-L44):
	bytes32 => bytes31 casting occurs in [bytes32(bytes.concat(new bytes(1),bytes31(sha256(bytes)(_data))))](src/core/libraries/Hash.sol#L43)
	bytes => bytes32 casting occurs in [bytes32(bytes.concat(new bytes(1),bytes31(sha256(bytes)(_data))))](src/core/libraries/Hash.sol#L43)

src/core/libraries/Hash.sol#L42-L44


 - [ ] ID-4
Dubious typecast in [TxsDecoder.read1(bytes,uint256)](src/core/libraries/decoders/TxsDecoder.sol#L334-L336):
	bytes => bytes1 casting occurs in [uint256(uint8(bytes1(slice(_data,_offset,1))))](src/core/libraries/decoders/TxsDecoder.sol#L335)

src/core/libraries/decoders/TxsDecoder.sol#L334-L336


 - [ ] ID-5
Dubious typecast in [TxsDecoder.read4(bytes,uint256)](src/core/libraries/decoders/TxsDecoder.sol#L344-L346):
	bytes => bytes4 casting occurs in [uint256(uint32(bytes4(slice(_data,_offset,4))))](src/core/libraries/decoders/TxsDecoder.sol#L345)

src/core/libraries/decoders/TxsDecoder.sol#L344-L346


 - [ ] ID-6
Dubious typecast in [HeaderLib.decode(bytes)](src/core/libraries/HeaderLib.sol#L143-L184):
	bytes => bytes32 casting occurs in [header.lastArchive = AppendOnlyTreeSnapshot(bytes32(_header),uint32(bytes4(_header)))](src/core/libraries/HeaderLib.sol#L151-L153)
	bytes => bytes4 casting occurs in [header.lastArchive = AppendOnlyTreeSnapshot(bytes32(_header),uint32(bytes4(_header)))](src/core/libraries/HeaderLib.sol#L151-L153)
	bytes => bytes32 casting occurs in [header.contentCommitment.txTreeHeight = uint256(bytes32(_header))](src/core/libraries/HeaderLib.sol#L156)
	bytes => bytes32 casting occurs in [header.contentCommitment.txsEffectsHash = bytes32(_header)](src/core/libraries/HeaderLib.sol#L157)
	bytes => bytes32 casting occurs in [header.contentCommitment.inHash = bytes32(_header)](src/core/libraries/HeaderLib.sol#L158)
	bytes => bytes32 casting occurs in [header.contentCommitment.outHash = bytes32(_header)](src/core/libraries/HeaderLib.sol#L159)
	bytes => bytes32 casting occurs in [header.stateReference.l1ToL2MessageTree = AppendOnlyTreeSnapshot(bytes32(_header),uint32(bytes4(_header)))](src/core/libraries/HeaderLib.sol#L162-L164)
	bytes => bytes4 casting occurs in [header.stateReference.l1ToL2MessageTree = AppendOnlyTreeSnapshot(bytes32(_header),uint32(bytes4(_header)))](src/core/libraries/HeaderLib.sol#L162-L164)
	bytes => bytes32 casting occurs in [header.stateReference.partialStateReference.noteHashTree = AppendOnlyTreeSnapshot(bytes32(_header),uint32(bytes4(_header)))](src/core/libraries/HeaderLib.sol#L165-L167)
	bytes => bytes4 casting occurs in [header.stateReference.partialStateReference.noteHashTree = AppendOnlyTreeSnapshot(bytes32(_header),uint32(bytes4(_header)))](src/core/libraries/HeaderLib.sol#L165-L167)
	bytes => bytes32 casting occurs in [header.stateReference.partialStateReference.nullifierTree = AppendOnlyTreeSnapshot(bytes32(_header),uint32(bytes4(_header)))](src/core/libraries/HeaderLib.sol#L168-L170)
	bytes => bytes4 casting occurs in [header.stateReference.partialStateReference.nullifierTree = AppendOnlyTreeSnapshot(bytes32(_header),uint32(bytes4(_header)))](src/core/libraries/HeaderLib.sol#L168-L170)
	bytes => bytes32 casting occurs in [header.stateReference.partialStateReference.publicDataTree = AppendOnlyTreeSnapshot(bytes32(_header),uint32(bytes4(_header)))](src/core/libraries/HeaderLib.sol#L171-L173)
	bytes => bytes4 casting occurs in [header.stateReference.partialStateReference.publicDataTree = AppendOnlyTreeSnapshot(bytes32(_header),uint32(bytes4(_header)))](src/core/libraries/HeaderLib.sol#L171-L173)
	bytes => bytes32 casting occurs in [header.globalVariables.chainId = uint256(bytes32(_header))](src/core/libraries/HeaderLib.sol#L176)
	bytes => bytes32 casting occurs in [header.globalVariables.version = uint256(bytes32(_header))](src/core/libraries/HeaderLib.sol#L177)
	bytes => bytes32 casting occurs in [header.globalVariables.blockNumber = uint256(bytes32(_header))](src/core/libraries/HeaderLib.sol#L178)
	bytes => bytes32 casting occurs in [header.globalVariables.timestamp = uint256(bytes32(_header))](src/core/libraries/HeaderLib.sol#L179)
	bytes => bytes20 casting occurs in [header.globalVariables.coinbase = address(bytes20(_header))](src/core/libraries/HeaderLib.sol#L180)
	bytes => bytes32 casting occurs in [header.globalVariables.feeRecipient = bytes32(_header)](src/core/libraries/HeaderLib.sol#L181)

src/core/libraries/HeaderLib.sol#L143-L184


## missing-zero-check
Impact: Low
Confidence: Medium
 - [ ] ID-7
[Inbox.constructor(address,uint256)._rollup](src/core/messagebridge/Inbox.sol#L40) lacks a zero-check on :
		- [ROLLUP = _rollup](src/core/messagebridge/Inbox.sol#L41)

src/core/messagebridge/Inbox.sol#L40


 - [ ] ID-8
[Outbox.constructor(address)._rollup](src/core/messagebridge/Outbox.sol#L31) lacks a zero-check on :
		- [ROLLUP_CONTRACT = _rollup](src/core/messagebridge/Outbox.sol#L32)

src/core/messagebridge/Outbox.sol#L31


## reentrancy-events
Impact: Low
Confidence: Medium
 - [ ] ID-9
Reentrancy in [Rollup.process(bytes,bytes32,bytes)](src/core/Rollup.sol#L58-L96):
	External calls:
	- [inHash = INBOX.consume()](src/core/Rollup.sol#L83)
	- [OUTBOX.insert(header.globalVariables.blockNumber,header.contentCommitment.outHash,l2ToL1TreeHeight)](src/core/Rollup.sol#L91-L93)
	Event emitted after the call(s):
	- [L2BlockProcessed(header.globalVariables.blockNumber)](src/core/Rollup.sol#L95)

src/core/Rollup.sol#L58-L96


 - [ ] ID-10
Reentrancy in [Inbox.sendL2Message(DataStructures.L2Actor,bytes32,bytes32)](src/core/messagebridge/Inbox.sol#L61-L95):
	External calls:
	- [index = currentTree.insertLeaf(leaf)](src/core/messagebridge/Inbox.sol#L91)
	Event emitted after the call(s):
	- [LeafInserted(inProgress,index,leaf)](src/core/messagebridge/Inbox.sol#L92)

src/core/messagebridge/Inbox.sol#L61-L95


## timestamp
Impact: Low
Confidence: Medium
 - [ ] ID-11
[HeaderLib.validate(HeaderLib.Header,uint256,uint256,bytes32)](src/core/libraries/HeaderLib.sol#L106-L136) uses timestamp for comparisons
	Dangerous comparisons:
	- [_header.globalVariables.timestamp > block.timestamp](src/core/libraries/HeaderLib.sol#L120)

src/core/libraries/HeaderLib.sol#L106-L136


## pess-public-vs-external
Impact: Low
Confidence: Medium
 - [ ] ID-12
The following public functions could be turned into external in [FrontierMerkle](src/core/messagebridge/frontier_tree/Frontier.sol#L12-L98) contract:
	[FrontierMerkle.constructor(uint256)](src/core/messagebridge/frontier_tree/Frontier.sol#L24-L32)

src/core/messagebridge/frontier_tree/Frontier.sol#L12-L98


 - [ ] ID-13
The following public functions could be turned into external in [Registry](src/core/messagebridge/Registry.sol#L22-L129) contract:
	[Registry.constructor()](src/core/messagebridge/Registry.sol#L29-L33)

src/core/messagebridge/Registry.sol#L22-L129


 - [ ] ID-14
The following public functions could be turned into external in [Inbox](src/core/messagebridge/Inbox.sol#L24-L124) contract:
	[Inbox.constructor(address,uint256)](src/core/messagebridge/Inbox.sol#L40-L51)

src/core/messagebridge/Inbox.sol#L24-L124


 - [ ] ID-15
The following public functions could be turned into external in [Rollup](src/core/Rollup.sol#L29-L105) contract:
	[Rollup.constructor(IRegistry,IAvailabilityOracle)](src/core/Rollup.sol#L43-L50)

src/core/Rollup.sol#L29-L105


 - [ ] ID-16
The following public functions could be turned into external in [Outbox](src/core/messagebridge/Outbox.sol#L18-L132) contract:
	[Outbox.constructor(address)](src/core/messagebridge/Outbox.sol#L31-L33)

src/core/messagebridge/Outbox.sol#L18-L132


## assembly
Impact: Informational
Confidence: High
 - [ ] ID-17
[TxsDecoder.computeRoot(bytes32[])](src/core/libraries/decoders/TxsDecoder.sol#L258-L277) uses assembly
	- [INLINE ASM](src/core/libraries/decoders/TxsDecoder.sol#L265-L267)

src/core/libraries/decoders/TxsDecoder.sol#L258-L277


## dead-code
Impact: Informational
Confidence: Medium
 - [ ] ID-18
[MessageBox.consume(mapping(bytes32 => DataStructures.Entry),bytes32,function(bytes32))](src/core/libraries/MessageBox.sol#L71-L79) is never used and should be removed

src/core/libraries/MessageBox.sol#L71-L79


 - [ ] ID-19
[MessageBox.contains(mapping(bytes32 => DataStructures.Entry),bytes32)](src/core/libraries/MessageBox.sol#L87-L92) is never used and should be removed

src/core/libraries/MessageBox.sol#L87-L92


 - [ ] ID-20
[MessageBox.get(mapping(bytes32 => DataStructures.Entry),bytes32,function(bytes32))](src/core/libraries/MessageBox.sol#L104-L112) is never used and should be removed

src/core/libraries/MessageBox.sol#L104-L112


 - [ ] ID-21
[MessageBox.insert(mapping(bytes32 => DataStructures.Entry),bytes32,uint64,uint32,uint32,function(bytes32,uint64,uint64,uint32,uint32,uint32,uint32))](src/core/libraries/MessageBox.sol#L30-L60) is never used and should be removed

src/core/libraries/MessageBox.sol#L30-L60


 - [ ] ID-22
[Hash.sha256ToField(bytes32)](src/core/libraries/Hash.sol#L52-L54) is never used and should be removed

src/core/libraries/Hash.sol#L52-L54


## solc-version
Impact: Informational
Confidence: High
 - [ ] ID-23
solc-0.8.23 is not recommended for deployment

## similar-names
Impact: Informational
Confidence: Medium
 - [ ] ID-24
Variable [Constants.LOGS_HASHES_NUM_BYTES_PER_BASE_ROLLUP](src/core/libraries/ConstantsGen.sol#L130) is too similar to [Constants.NOTE_HASHES_NUM_BYTES_PER_BASE_ROLLUP](src/core/libraries/ConstantsGen.sol#L123)

src/core/libraries/ConstantsGen.sol#L130


 - [ ] ID-25
Variable [Constants.L1_TO_L2_MESSAGE_LENGTH](src/core/libraries/ConstantsGen.sol#L110) is too similar to [Constants.L2_TO_L1_MESSAGE_LENGTH](src/core/libraries/ConstantsGen.sol#L111)

src/core/libraries/ConstantsGen.sol#L110


 - [ ] ID-26
Variable [Rollup.AVAILABILITY_ORACLE](src/core/Rollup.sol#L32) is too similar to [Rollup.constructor(IRegistry,IAvailabilityOracle)._availabilityOracle](src/core/Rollup.sol#L43)

src/core/Rollup.sol#L32


## constable-states
Impact: Optimization
Confidence: High
 - [ ] ID-27
[Rollup.lastWarpedBlockTs](src/core/Rollup.sol#L41) should be constant 

src/core/Rollup.sol#L41


## pess-multiple-storage-read
Impact: Optimization
Confidence: High
 - [ ] ID-28
In a function [Outbox.insert(uint256,bytes32,uint256)](src/core/messagebridge/Outbox.sol#L44-L64) variable [Outbox.roots](src/core/messagebridge/Outbox.sol#L29) is read multiple times

src/core/messagebridge/Outbox.sol#L44-L64


 - [ ] ID-29
In a function [Inbox.consume()](src/core/messagebridge/Inbox.sol#L104-L123) variable [Inbox.toConsume](src/core/messagebridge/Inbox.sol#L34) is read multiple times

src/core/messagebridge/Inbox.sol#L104-L123


 - [ ] ID-30
In a function [Inbox.consume()](src/core/messagebridge/Inbox.sol#L104-L123) variable [Inbox.inProgress](src/core/messagebridge/Inbox.sol#L36) is read multiple times

src/core/messagebridge/Inbox.sol#L104-L123


 - [ ] ID-31
In a function [FrontierMerkle.root()](src/core/messagebridge/frontier_tree/Frontier.sol#L48-L81) variable [FrontierMerkle.HEIGHT](src/core/messagebridge/frontier_tree/Frontier.sol#L13) is read multiple times

src/core/messagebridge/frontier_tree/Frontier.sol#L48-L81


 - [ ] ID-32
In a function [Inbox.sendL2Message(DataStructures.L2Actor,bytes32,bytes32)](src/core/messagebridge/Inbox.sol#L61-L95) variable [Inbox.inProgress](src/core/messagebridge/Inbox.sol#L36) is read multiple times

src/core/messagebridge/Inbox.sol#L61-L95


 - [ ] ID-33
In a function [FrontierMerkle.root()](src/core/messagebridge/frontier_tree/Frontier.sol#L48-L81) variable [FrontierMerkle.frontier](src/core/messagebridge/frontier_tree/Frontier.sol#L18) is read multiple times

src/core/messagebridge/frontier_tree/Frontier.sol#L48-L81


