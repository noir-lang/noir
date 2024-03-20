Summary
 - [pess-unprotected-setter](#pess-unprotected-setter) (1 results) (High)
 - [uninitialized-local](#uninitialized-local) (2 results) (Medium)
 - [unused-return](#unused-return) (1 results) (Medium)
 - [pess-dubious-typecast](#pess-dubious-typecast) (7 results) (Medium)
 - [missing-zero-check](#missing-zero-check) (2 results) (Low)
 - [reentrancy-events](#reentrancy-events) (2 results) (Low)
 - [timestamp](#timestamp) (1 results) (Low)
 - [pess-public-vs-external](#pess-public-vs-external) (6 results) (Low)
 - [assembly](#assembly) (2 results) (Informational)
 - [dead-code](#dead-code) (3 results) (Informational)
 - [solc-version](#solc-version) (1 results) (Informational)
 - [similar-names](#similar-names) (3 results) (Informational)
 - [constable-states](#constable-states) (1 results) (Optimization)
 - [pess-multiple-storage-read](#pess-multiple-storage-read) (6 results) (Optimization)
## pess-unprotected-setter
Impact: High
Confidence: Medium
 - [ ] ID-0
Function [Rollup.process(bytes,bytes32,bytes,bytes)](src/core/Rollup.sol#L57-L96) is a non-protected setter archive is written

src/core/Rollup.sol#L57-L96


## uninitialized-local
Impact: Medium
Confidence: Medium
 - [ ] ID-1
[HeaderLib.decode(bytes).header](src/core/libraries/HeaderLib.sol#L148) is a local variable never initialized

src/core/libraries/HeaderLib.sol#L148


 - [ ] ID-2
[TxsDecoder.decode(bytes).vars](src/core/libraries/decoders/TxsDecoder.sol#L81) is a local variable never initialized

src/core/libraries/decoders/TxsDecoder.sol#L81


## unused-return
Impact: Medium
Confidence: Medium
 - [ ] ID-3
[Rollup.process(bytes,bytes32,bytes,bytes)](src/core/Rollup.sol#L57-L96) ignores return value by [(l2ToL1Msgs) = MessagesDecoder.decode(_body)](src/core/Rollup.sol#L73)

src/core/Rollup.sol#L57-L96


## pess-dubious-typecast
Impact: Medium
Confidence: High
 - [ ] ID-4
Dubious typecast in [MessagesDecoder.read4(bytes,uint256)](src/core/libraries/decoders/MessagesDecoder.sol#L164-L166):
	bytes => bytes4 casting occurs in [uint256(uint32(bytes4(_data)))](src/core/libraries/decoders/MessagesDecoder.sol#L165)

src/core/libraries/decoders/MessagesDecoder.sol#L164-L166


 - [ ] ID-5
Dubious typecast in [Outbox.sendL1Messages(bytes32[])](src/core/messagebridge/Outbox.sol#L38-L46):
	uint256 => uint32 casting occurs in [version = uint32(REGISTRY.getVersionFor(msg.sender))](src/core/messagebridge/Outbox.sol#L40)

src/core/messagebridge/Outbox.sol#L38-L46


 - [ ] ID-6
Dubious typecast in [TxsDecoder.read1(bytes,uint256)](src/core/libraries/decoders/TxsDecoder.sol#L322-L324):
	bytes => bytes1 casting occurs in [uint256(uint8(bytes1(slice(_data,_offset,1))))](src/core/libraries/decoders/TxsDecoder.sol#L323)

src/core/libraries/decoders/TxsDecoder.sol#L322-L324


 - [ ] ID-7
Dubious typecast in [TxsDecoder.read4(bytes,uint256)](src/core/libraries/decoders/TxsDecoder.sol#L332-L334):
	bytes => bytes4 casting occurs in [uint256(uint32(bytes4(slice(_data,_offset,4))))](src/core/libraries/decoders/TxsDecoder.sol#L333)

src/core/libraries/decoders/TxsDecoder.sol#L332-L334


 - [ ] ID-8
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


 - [ ] ID-9
Dubious typecast in [TxsDecoder.decode(bytes)](src/core/libraries/decoders/TxsDecoder.sol#L78-L192):
	bytes => bytes32 casting occurs in [vars.baseLeaf = bytes.concat(bytes32(slice(_body,offsets.revertCode,0x20)),bytes.concat(sliceAndPad(_body,offsets.noteHash,counts.noteHash * 0x20,Constants.NOTE_HASHES_NUM_BYTES_PER_BASE_ROLLUP),sliceAndPad(_body,offsets.nullifier,counts.nullifier * 0x20,Constants.NULLIFIERS_NUM_BYTES_PER_BASE_ROLLUP),sliceAndPad(_body,offsets.l2ToL1Msgs,counts.l2ToL1Msgs * 0x20,Constants.L2_TO_L1_MSGS_NUM_BYTES_PER_BASE_ROLLUP),sliceAndPad(_body,offsets.publicData,counts.publicData * 0x40,Constants.PUBLIC_DATA_WRITES_NUM_BYTES_PER_BASE_ROLLUP)),bytes.concat(vars.encryptedLogsHash,vars.unencryptedLogsHash))](src/core/libraries/decoders/TxsDecoder.sol#L156-L185)

src/core/libraries/decoders/TxsDecoder.sol#L78-L192


 - [ ] ID-10
Dubious typecast in [MessagesDecoder.read1(bytes,uint256)](src/core/libraries/decoders/MessagesDecoder.sol#L154-L156):
	bytes => bytes1 casting occurs in [uint256(uint8(bytes1(_data)))](src/core/libraries/decoders/MessagesDecoder.sol#L155)

src/core/libraries/decoders/MessagesDecoder.sol#L154-L156


## missing-zero-check
Impact: Low
Confidence: Medium
 - [ ] ID-11
[Inbox.constructor(address,uint256)._rollup](src/core/messagebridge/Inbox.sol#L40) lacks a zero-check on :
		- [ROLLUP = _rollup](src/core/messagebridge/Inbox.sol#L41)

src/core/messagebridge/Inbox.sol#L40


 - [ ] ID-12
[NewOutbox.constructor(address)._rollup](src/core/messagebridge/NewOutbox.sol#L31) lacks a zero-check on :
		- [ROLLUP_CONTRACT = _rollup](src/core/messagebridge/NewOutbox.sol#L32)

src/core/messagebridge/NewOutbox.sol#L31


## reentrancy-events
Impact: Low
Confidence: Medium
 - [ ] ID-13
Reentrancy in [Inbox.sendL2Message(DataStructures.L2Actor,bytes32,bytes32)](src/core/messagebridge/Inbox.sol#L61-L95):
	External calls:
	- [index = currentTree.insertLeaf(leaf)](src/core/messagebridge/Inbox.sol#L91)
	Event emitted after the call(s):
	- [LeafInserted(inProgress,index,leaf)](src/core/messagebridge/Inbox.sol#L92)

src/core/messagebridge/Inbox.sol#L61-L95


 - [ ] ID-14
Reentrancy in [Rollup.process(bytes,bytes32,bytes,bytes)](src/core/Rollup.sol#L57-L96):
	External calls:
	- [inHash = INBOX.consume()](src/core/Rollup.sol#L87)
	- [outbox.sendL1Messages(l2ToL1Msgs)](src/core/Rollup.sol#L93)
	Event emitted after the call(s):
	- [L2BlockProcessed(header.globalVariables.blockNumber)](src/core/Rollup.sol#L95)

src/core/Rollup.sol#L57-L96


## timestamp
Impact: Low
Confidence: Medium
 - [ ] ID-15
[HeaderLib.validate(HeaderLib.Header,uint256,uint256,bytes32)](src/core/libraries/HeaderLib.sol#L106-L136) uses timestamp for comparisons
	Dangerous comparisons:
	- [_header.globalVariables.timestamp > block.timestamp](src/core/libraries/HeaderLib.sol#L120)

src/core/libraries/HeaderLib.sol#L106-L136


## pess-public-vs-external
Impact: Low
Confidence: Medium
 - [ ] ID-16
The following public functions could be turned into external in [FrontierMerkle](src/core/messagebridge/frontier_tree/Frontier.sol#L7-L93) contract:
	[FrontierMerkle.constructor(uint256)](src/core/messagebridge/frontier_tree/Frontier.sol#L19-L27)

src/core/messagebridge/frontier_tree/Frontier.sol#L7-L93


 - [ ] ID-17
The following public functions could be turned into external in [Registry](src/core/messagebridge/Registry.sol#L22-L129) contract:
	[Registry.constructor()](src/core/messagebridge/Registry.sol#L29-L33)

src/core/messagebridge/Registry.sol#L22-L129


 - [ ] ID-18
The following public functions could be turned into external in [Inbox](src/core/messagebridge/Inbox.sol#L24-L124) contract:
	[Inbox.constructor(address,uint256)](src/core/messagebridge/Inbox.sol#L40-L51)

src/core/messagebridge/Inbox.sol#L24-L124


 - [ ] ID-19
The following public functions could be turned into external in [Rollup](src/core/Rollup.sol#L29-L105) contract:
	[Rollup.constructor(IRegistry,IAvailabilityOracle)](src/core/Rollup.sol#L42-L48)

src/core/Rollup.sol#L29-L105


 - [ ] ID-20
The following public functions could be turned into external in [Outbox](src/core/messagebridge/Outbox.sol#L21-L148) contract:
	[Outbox.constructor(address)](src/core/messagebridge/Outbox.sol#L29-L31)
	[Outbox.get(bytes32)](src/core/messagebridge/Outbox.sol#L77-L84)
	[Outbox.contains(bytes32)](src/core/messagebridge/Outbox.sol#L91-L93)

src/core/messagebridge/Outbox.sol#L21-L148


 - [ ] ID-21
The following public functions could be turned into external in [NewOutbox](src/core/messagebridge/NewOutbox.sol#L18-L132) contract:
	[NewOutbox.constructor(address)](src/core/messagebridge/NewOutbox.sol#L31-L33)

src/core/messagebridge/NewOutbox.sol#L18-L132


## assembly
Impact: Informational
Confidence: High
 - [ ] ID-22
[MessagesDecoder.decode(bytes)](src/core/libraries/decoders/MessagesDecoder.sol#L61-L146) uses assembly
	- [INLINE ASM](src/core/libraries/decoders/MessagesDecoder.sol#L80-L82)
	- [INLINE ASM](src/core/libraries/decoders/MessagesDecoder.sol#L116-L122)

src/core/libraries/decoders/MessagesDecoder.sol#L61-L146


 - [ ] ID-23
[TxsDecoder.computeRoot(bytes32[])](src/core/libraries/decoders/TxsDecoder.sol#L264-L283) uses assembly
	- [INLINE ASM](src/core/libraries/decoders/TxsDecoder.sol#L271-L273)

src/core/libraries/decoders/TxsDecoder.sol#L264-L283


## dead-code
Impact: Informational
Confidence: Medium
 - [ ] ID-24
[Outbox._errNothingToConsume(bytes32)](src/core/messagebridge/Outbox.sol#L114-L116) is never used and should be removed

src/core/messagebridge/Outbox.sol#L114-L116


 - [ ] ID-25
[Hash.sha256ToField(bytes32)](src/core/libraries/Hash.sol#L52-L54) is never used and should be removed

src/core/libraries/Hash.sol#L52-L54


 - [ ] ID-26
[Outbox._errIncompatibleEntryArguments(bytes32,uint64,uint64,uint32,uint32,uint32,uint32)](src/core/messagebridge/Outbox.sol#L129-L147) is never used and should be removed

src/core/messagebridge/Outbox.sol#L129-L147


## solc-version
Impact: Informational
Confidence: High
 - [ ] ID-27
solc-0.8.23 is not recommended for deployment

## similar-names
Impact: Informational
Confidence: Medium
 - [ ] ID-28
Variable [Constants.LOGS_HASHES_NUM_BYTES_PER_BASE_ROLLUP](src/core/libraries/ConstantsGen.sol#L130) is too similar to [Constants.NOTE_HASHES_NUM_BYTES_PER_BASE_ROLLUP](src/core/libraries/ConstantsGen.sol#L123)

src/core/libraries/ConstantsGen.sol#L130


 - [ ] ID-29
Variable [Constants.L1_TO_L2_MESSAGE_LENGTH](src/core/libraries/ConstantsGen.sol#L110) is too similar to [Constants.L2_TO_L1_MESSAGE_LENGTH](src/core/libraries/ConstantsGen.sol#L111)

src/core/libraries/ConstantsGen.sol#L110


 - [ ] ID-30
Variable [Rollup.AVAILABILITY_ORACLE](src/core/Rollup.sol#L32) is too similar to [Rollup.constructor(IRegistry,IAvailabilityOracle)._availabilityOracle](src/core/Rollup.sol#L42)

src/core/Rollup.sol#L32


## constable-states
Impact: Optimization
Confidence: High
 - [ ] ID-31
[Rollup.lastWarpedBlockTs](src/core/Rollup.sol#L40) should be constant 

src/core/Rollup.sol#L40


## pess-multiple-storage-read
Impact: Optimization
Confidence: High
 - [ ] ID-32
In a function [NewOutbox.insert(uint256,bytes32,uint256)](src/core/messagebridge/NewOutbox.sol#L44-L64) variable [NewOutbox.roots](src/core/messagebridge/NewOutbox.sol#L29) is read multiple times

src/core/messagebridge/NewOutbox.sol#L44-L64


 - [ ] ID-33
In a function [Inbox.consume()](src/core/messagebridge/Inbox.sol#L104-L123) variable [Inbox.toConsume](src/core/messagebridge/Inbox.sol#L34) is read multiple times

src/core/messagebridge/Inbox.sol#L104-L123


 - [ ] ID-34
In a function [Inbox.consume()](src/core/messagebridge/Inbox.sol#L104-L123) variable [Inbox.inProgress](src/core/messagebridge/Inbox.sol#L36) is read multiple times

src/core/messagebridge/Inbox.sol#L104-L123


 - [ ] ID-35
In a function [FrontierMerkle.root()](src/core/messagebridge/frontier_tree/Frontier.sol#L43-L76) variable [FrontierMerkle.HEIGHT](src/core/messagebridge/frontier_tree/Frontier.sol#L8) is read multiple times

src/core/messagebridge/frontier_tree/Frontier.sol#L43-L76


 - [ ] ID-36
In a function [Inbox.sendL2Message(DataStructures.L2Actor,bytes32,bytes32)](src/core/messagebridge/Inbox.sol#L61-L95) variable [Inbox.inProgress](src/core/messagebridge/Inbox.sol#L36) is read multiple times

src/core/messagebridge/Inbox.sol#L61-L95


 - [ ] ID-37
In a function [FrontierMerkle.root()](src/core/messagebridge/frontier_tree/Frontier.sol#L43-L76) variable [FrontierMerkle.frontier](src/core/messagebridge/frontier_tree/Frontier.sol#L13) is read multiple times

src/core/messagebridge/frontier_tree/Frontier.sol#L43-L76


