Summary
 - [uninitialized-local](#uninitialized-local) (1 results) (Medium)
 - [unused-return](#unused-return) (1 results) (Medium)
 - [reentrancy-events](#reentrancy-events) (1 results) (Low)
 - [timestamp](#timestamp) (4 results) (Low)
 - [assembly](#assembly) (5 results) (Informational)
 - [dead-code](#dead-code) (13 results) (Informational)
 - [solc-version](#solc-version) (1 results) (Informational)
 - [low-level-calls](#low-level-calls) (1 results) (Informational)
 - [similar-names](#similar-names) (1 results) (Informational)
 - [unused-state](#unused-state) (2 results) (Informational)
 - [constable-states](#constable-states) (1 results) (Optimization)
## uninitialized-local
Impact: Medium
Confidence: Medium
 - [ ] ID-0
[HeaderLib.decode(bytes).header](src/core/libraries/HeaderLib.sol#L133) is a local variable never initialized

src/core/libraries/HeaderLib.sol#L133


## unused-return
Impact: Medium
Confidence: Medium
 - [ ] ID-1
[Rollup.process(bytes,bytes32,bytes32,bytes,bytes)](src/core/Rollup.sol#L54-L94) ignores return value by [(inHash,l1ToL2Msgs,l2ToL1Msgs) = MessagesDecoder.decode(_body)](src/core/Rollup.sol#L71-L72)

src/core/Rollup.sol#L54-L94


## reentrancy-events
Impact: Low
Confidence: Medium
 - [ ] ID-2
Reentrancy in [Rollup.process(bytes,bytes32,bytes32,bytes,bytes)](src/core/Rollup.sol#L54-L94):
	External calls:
	- [inbox.batchConsume(l1ToL2Msgs,msg.sender)](src/core/Rollup.sol#L88)
	- [outbox.sendL1Messages(l2ToL1Msgs)](src/core/Rollup.sol#L91)
	Event emitted after the call(s):
	- [L2BlockProcessed(header.globalVariables.blockNumber)](src/core/Rollup.sol#L93)

src/core/Rollup.sol#L54-L94


## timestamp
Impact: Low
Confidence: Medium
 - [ ] ID-3
[Inbox.batchConsume(bytes32[],address)](src/core/messagebridge/Inbox.sol#L122-L143) uses timestamp for comparisons
	Dangerous comparisons:
	- [block.timestamp > entry.deadline](src/core/messagebridge/Inbox.sol#L136)

src/core/messagebridge/Inbox.sol#L122-L143


 - [ ] ID-4
[Inbox.sendL2Message(DataStructures.L2Actor,uint32,bytes32,bytes32)](src/core/messagebridge/Inbox.sol#L45-L91) uses timestamp for comparisons
	Dangerous comparisons:
	- [_deadline <= block.timestamp](src/core/messagebridge/Inbox.sol#L54)

src/core/messagebridge/Inbox.sol#L45-L91


 - [ ] ID-5
[HeaderLib.validate(HeaderLib.Header,uint256,uint256,bytes32)](src/core/libraries/HeaderLib.sol#L91-L121) uses timestamp for comparisons
	Dangerous comparisons:
	- [_header.globalVariables.timestamp > block.timestamp](src/core/libraries/HeaderLib.sol#L105)

src/core/libraries/HeaderLib.sol#L91-L121


 - [ ] ID-6
[Inbox.cancelL2Message(DataStructures.L1ToL2Msg,address)](src/core/messagebridge/Inbox.sol#L102-L113) uses timestamp for comparisons
	Dangerous comparisons:
	- [block.timestamp <= _message.deadline](src/core/messagebridge/Inbox.sol#L108)

src/core/messagebridge/Inbox.sol#L102-L113


## assembly
Impact: Informational
Confidence: High
 - [ ] ID-7
[Decoder.computeRoot(bytes32[])](src/core/libraries/decoders/Decoder.sol#L373-L392) uses assembly
	- [INLINE ASM](src/core/libraries/decoders/Decoder.sol#L380-L382)

src/core/libraries/decoders/Decoder.sol#L373-L392


 - [ ] ID-8
[TxsDecoder.decode(bytes)](src/core/libraries/decoders/TxsDecoder.sol#L71-L184) uses assembly
	- [INLINE ASM](src/core/libraries/decoders/TxsDecoder.sol#L98-L104)

src/core/libraries/decoders/TxsDecoder.sol#L71-L184


 - [ ] ID-9
[Decoder.computeConsumables(bytes)](src/core/libraries/decoders/Decoder.sol#L164-L301) uses assembly
	- [INLINE ASM](src/core/libraries/decoders/Decoder.sol#L196-L202)
	- [INLINE ASM](src/core/libraries/decoders/Decoder.sol#L289-L295)

src/core/libraries/decoders/Decoder.sol#L164-L301


 - [ ] ID-10
[TxsDecoder.computeRoot(bytes32[])](src/core/libraries/decoders/TxsDecoder.sol#L256-L275) uses assembly
	- [INLINE ASM](src/core/libraries/decoders/TxsDecoder.sol#L263-L265)

src/core/libraries/decoders/TxsDecoder.sol#L256-L275


 - [ ] ID-11
[MessagesDecoder.decode(bytes)](src/core/libraries/decoders/MessagesDecoder.sol#L52-L102) uses assembly
	- [INLINE ASM](src/core/libraries/decoders/MessagesDecoder.sol#L81-L83)
	- [INLINE ASM](src/core/libraries/decoders/MessagesDecoder.sol#L94-L96)

src/core/libraries/decoders/MessagesDecoder.sol#L52-L102


## dead-code
Impact: Informational
Confidence: Medium
 - [ ] ID-12
[Decoder.computeConsumables(bytes)](src/core/libraries/decoders/Decoder.sol#L164-L301) is never used and should be removed

src/core/libraries/decoders/Decoder.sol#L164-L301


 - [ ] ID-13
[Inbox._errIncompatibleEntryArguments(bytes32,uint64,uint64,uint32,uint32,uint32,uint32)](src/core/messagebridge/Inbox.sol#L212-L230) is never used and should be removed

src/core/messagebridge/Inbox.sol#L212-L230


 - [ ] ID-14
[Decoder.slice(bytes,uint256,uint256)](src/core/libraries/decoders/Decoder.sol#L401-L407) is never used and should be removed

src/core/libraries/decoders/Decoder.sol#L401-L407


 - [ ] ID-15
[Outbox._errNothingToConsume(bytes32)](src/core/messagebridge/Outbox.sol#L115-L117) is never used and should be removed

src/core/messagebridge/Outbox.sol#L115-L117


 - [ ] ID-16
[Decoder.computeRoot(bytes32[])](src/core/libraries/decoders/Decoder.sol#L373-L392) is never used and should be removed

src/core/libraries/decoders/Decoder.sol#L373-L392


 - [ ] ID-17
[Hash.sha256ToField(bytes32)](src/core/libraries/Hash.sol#L59-L61) is never used and should be removed

src/core/libraries/Hash.sol#L59-L61


 - [ ] ID-18
[Decoder.computeKernelLogsHash(uint256,bytes)](src/core/libraries/decoders/Decoder.sol#L335-L365) is never used and should be removed

src/core/libraries/decoders/Decoder.sol#L335-L365


 - [ ] ID-19
[Decoder.read4(bytes,uint256)](src/core/libraries/decoders/Decoder.sol#L415-L417) is never used and should be removed

src/core/libraries/decoders/Decoder.sol#L415-L417


 - [ ] ID-20
[Decoder.computeStateHash(uint256,uint256,bytes)](src/core/libraries/decoders/Decoder.sol#L146-L154) is never used and should be removed

src/core/libraries/decoders/Decoder.sol#L146-L154


 - [ ] ID-21
[Decoder.computePublicInputHash(bytes,bytes32,bytes32)](src/core/libraries/decoders/Decoder.sol#L118-L125) is never used and should be removed

src/core/libraries/decoders/Decoder.sol#L118-L125


 - [ ] ID-22
[Inbox._errNothingToConsume(bytes32)](src/core/messagebridge/Inbox.sol#L197-L199) is never used and should be removed

src/core/messagebridge/Inbox.sol#L197-L199


 - [ ] ID-23
[Decoder.getL2BlockNumber(bytes)](src/core/libraries/decoders/Decoder.sol#L132-L134) is never used and should be removed

src/core/libraries/decoders/Decoder.sol#L132-L134


 - [ ] ID-24
[Outbox._errIncompatibleEntryArguments(bytes32,uint64,uint64,uint32,uint32,uint32,uint32)](src/core/messagebridge/Outbox.sol#L130-L148) is never used and should be removed

src/core/messagebridge/Outbox.sol#L130-L148


## solc-version
Impact: Informational
Confidence: High
 - [ ] ID-25
solc-0.8.21 is not recommended for deployment

## low-level-calls
Impact: Informational
Confidence: High
 - [ ] ID-26
Low level call in [Inbox.withdrawFees()](src/core/messagebridge/Inbox.sol#L148-L153):
	- [(success) = msg.sender.call{value: balance}()](src/core/messagebridge/Inbox.sol#L151)

src/core/messagebridge/Inbox.sol#L148-L153


## similar-names
Impact: Informational
Confidence: Medium
 - [ ] ID-27
Variable [Rollup.AVAILABILITY_ORACLE](src/core/Rollup.sol#L30) is too similar to [Rollup.constructor(IRegistry,IAvailabilityOracle)._availabilityOracle](src/core/Rollup.sol#L39)

src/core/Rollup.sol#L30


## unused-state
Impact: Informational
Confidence: High
 - [ ] ID-28
[Decoder.END_TREES_BLOCK_HEADER_OFFSET](src/core/libraries/decoders/Decoder.sol#L103-L104) is never used in [Decoder](src/core/libraries/decoders/Decoder.sol#L72-L418)

src/core/libraries/decoders/Decoder.sol#L103-L104


 - [ ] ID-29
[Decoder.BLOCK_HEADER_OFFSET](src/core/libraries/decoders/Decoder.sol#L107-L108) is never used in [Decoder](src/core/libraries/decoders/Decoder.sol#L72-L418)

src/core/libraries/decoders/Decoder.sol#L107-L108


## constable-states
Impact: Optimization
Confidence: High
 - [ ] ID-30
[Rollup.lastWarpedBlockTs](src/core/Rollup.sol#L37) should be constant 

src/core/Rollup.sol#L37


