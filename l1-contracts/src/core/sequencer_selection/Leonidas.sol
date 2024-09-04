// SPDX-License-Identifier: Apache-2.0
// Copyright 2024 Aztec Labs.
pragma solidity >=0.8.18;

import {DataStructures} from "../libraries/DataStructures.sol";
import {Errors} from "../libraries/Errors.sol";
import {EnumerableSet} from "@oz/utils/structs/EnumerableSet.sol";
import {Ownable} from "@oz/access/Ownable.sol";
import {SignatureLib} from "./SignatureLib.sol";
import {SampleLib} from "./SampleLib.sol";
import {Constants} from "../libraries/ConstantsGen.sol";
import {MessageHashUtils} from "@oz/utils/cryptography/MessageHashUtils.sol";

import {ILeonidas} from "./ILeonidas.sol";

/**
 * @title   Leonidas
 * @author  Anaxandridas II
 * @notice  Leonidas is the spartan king, it is his job to select the warriors progressing the state of the kingdom.
 *          He define the structure needed for committee and leader selection and provides logic for validating that
 *          the block and its "evidence" follows his rules.
 *
 * @dev     Leonidas is depending on Ares to add/remove warriors to/from his army competently.
 *
 * @dev     Leonidas have one thing in mind, he provide a reference of the LOGIC going on for the spartan selection.
 *          He is not concerned about gas costs, he is a king, he just throw gas in the air like no-one cares.
 *          It will be the duty of his successor (Pleistarchus) to optimize the costs with same functionality.
 *
 */
contract Leonidas is Ownable, ILeonidas {
  using EnumerableSet for EnumerableSet.AddressSet;
  using SignatureLib for SignatureLib.Signature;
  using MessageHashUtils for bytes32;

  /**
   * @notice  The data structure for an epoch
   * @param committee - The validator set for the epoch
   * @param sampleSeed - The seed used to sample the validator set of the epoch
   * @param nextSeed - The seed used to influence the NEXT epoch
   */
  struct Epoch {
    address[] committee;
    uint256 sampleSeed;
    uint256 nextSeed;
  }

  // @note  @LHerskind  The multiple cause pain and suffering in the E2E tests as we introduce
  //                    a timeliness requirement into the publication that did not exists before,
  //                    and at the same time have a setup that will impact the time at every tx
  //                    because of auto-mine. By using just 1, we can make our test work
  //                    but anything using an actual working chain would eat dung as simulating
  //                    transactions is slower than an actual ethereum slot.
  //
  //                    The value should be a higher multiple for any actual chain
  // @todo  #8019
  uint256 public constant SLOT_DURATION = Constants.AZTEC_SLOT_DURATION;

  // The duration of an epoch in slots
  // @todo  @LHerskind - This value should be updated when we are not blind.
  // @todo  #8020
  uint256 public constant EPOCH_DURATION = Constants.AZTEC_EPOCH_DURATION;

  // The target number of validators in a committee
  // @todo #8021
  uint256 public constant TARGET_COMMITTEE_SIZE = Constants.AZTEC_TARGET_COMMITTEE_SIZE;

  // The time that the contract was deployed
  uint256 public immutable GENESIS_TIME;

  // An enumerable set of validators that are up to date
  EnumerableSet.AddressSet private validatorSet;

  // A mapping to snapshots of the validator set
  mapping(uint256 epochNumber => Epoch epoch) public epochs;

  // The last stored randao value, same value as `seed` in the last inserted epoch
  uint256 private lastSeed;

  constructor(address _ares) Ownable(_ares) {
    GENESIS_TIME = block.timestamp;
  }

  /**
   * @notice  Adds a validator to the validator set
   *
   * @dev     Only ARES can add validators
   *
   * @dev     Will setup the epoch if needed BEFORE adding the validator.
   *          This means that the validator will effectively be added to the NEXT epoch.
   *
   * @param _validator - The validator to add
   */
  function addValidator(address _validator) external override(ILeonidas) onlyOwner {
    setupEpoch();
    _addValidator(_validator);
  }

  /**
   * @notice  Removes a validator from the validator set
   *
   * @dev     Only ARES can add validators
   *
   * @dev     Will setup the epoch if needed BEFORE removing the validator.
   *          This means that the validator will effectively be removed from the NEXT epoch.
   *
   * @param _validator - The validator to remove
   */
  function removeValidator(address _validator) external override(ILeonidas) onlyOwner {
    setupEpoch();
    validatorSet.remove(_validator);
  }

  /**
   * @notice  Get the validator set for a given epoch
   *
   * @dev     Consider removing this to replace with a `size` and individual getter.
   *
   * @param _epoch The epoch number to get the validator set for
   *
   * @return The validator set for the given epoch
   */
  function getEpochCommittee(uint256 _epoch)
    external
    view
    override(ILeonidas)
    returns (address[] memory)
  {
    return epochs[_epoch].committee;
  }

  function getCommitteeAt(uint256 _ts) internal view returns (address[] memory) {
    uint256 epochNumber = getEpochAt(_ts);
    Epoch storage epoch = epochs[epochNumber];

    if (epoch.sampleSeed != 0) {
      uint256 committeeSize = epoch.committee.length;
      if (committeeSize == 0) {
        return new address[](0);
      }
      return epoch.committee;
    }

    // Allow anyone if there is no validator set
    if (validatorSet.length() == 0) {
      return new address[](0);
    }

    // Emulate a sampling of the validators
    uint256 sampleSeed = _getSampleSeed(epochNumber);
    return _sampleValidators(sampleSeed);
  }

  /**
   * @notice  Get the validator set for the current epoch
   *
   * @dev Makes a call to setupEpoch under the hood, this should ONLY be called as a view function, and not from within
   *      this contract.
   * @return The validator set for the current epoch
   */
  function getCurrentEpochCommittee() external view override(ILeonidas) returns (address[] memory) {
    return getCommitteeAt(block.timestamp);
  }

  /**
   * @notice  Get the validator set
   *
   * @dev     Consider removing this to replace with a `size` and individual getter.
   *
   * @return The validator set
   */
  function getValidators() external view override(ILeonidas) returns (address[] memory) {
    return validatorSet.values();
  }

  /**
   * @notice  Get the number of validators in the validator set
   *
   * @return The number of validators in the validator set
   */
  function getValidatorCount() public view override(ILeonidas) returns (uint256) {
    return validatorSet.length();
  }

  /**
   * @notice  Get the number of validators in the validator set
   *
   * @return The number of validators in the validator set
   */
  function getValidatorAt(uint256 _index) public view override(ILeonidas) returns (address) {
    return validatorSet.at(_index);
  }

  /**
   * @notice  Checks if an address is in the validator set
   *
   * @param _validator - The address to check
   *
   * @return True if the address is in the validator set, false otherwise
   */
  function isValidator(address _validator) public view override(ILeonidas) returns (bool) {
    return validatorSet.contains(_validator);
  }

  /**
   * @notice  Performs a setup of an epoch if needed. The setup will
   *          - Sample the validator set for the epoch
   *          - Set the seed for the epoch
   *          - Update the last seed
   *
   * @dev     Since this is a reference optimising for simplicity, we store the actual validator set in the epoch structure.
   *          This is very heavy on gas, so start crying because the gas here will melt the poles
   *          https://i.giphy.com/U1aN4HTfJ2SmgB2BBK.webp
   */
  function setupEpoch() public override(ILeonidas) {
    uint256 epochNumber = getCurrentEpoch();
    Epoch storage epoch = epochs[epochNumber];

    if (epoch.sampleSeed == 0) {
      epoch.sampleSeed = _getSampleSeed(epochNumber);
      epoch.nextSeed = lastSeed = _computeNextSeed(epochNumber);

      epoch.committee = _sampleValidators(epoch.sampleSeed);
    }
  }

  /**
   * @notice  Get the current epoch number
   *
   * @return The current epoch number
   */
  function getCurrentEpoch() public view override(ILeonidas) returns (uint256) {
    return getEpochAt(block.timestamp);
  }

  /**
   * @notice  Get the current slot number
   *
   * @return The current slot number
   */
  function getCurrentSlot() public view override(ILeonidas) returns (uint256) {
    return getSlotAt(block.timestamp);
  }

  /**
   * @notice  Get the timestamp for a given slot
   *
   * @param _slotNumber - The slot number to get the timestamp for
   *
   * @return The timestamp for the given slot
   */
  function getTimestampForSlot(uint256 _slotNumber)
    public
    view
    override(ILeonidas)
    returns (uint256)
  {
    return _slotNumber * SLOT_DURATION + GENESIS_TIME;
  }

  /**
   * @notice  Get the proposer for the current slot
   *
   * @dev     Calls `getCurrentProposer(uint256)` with the current timestamp
   *
   * @return The address of the proposer
   */
  function getCurrentProposer() public view override(ILeonidas) returns (address) {
    return getProposerAt(block.timestamp);
  }

  /**
   * @notice  Get the proposer for the slot at a specific timestamp
   *
   * @dev     This function is very useful for off-chain usage, as it easily allow a client to
   *          determine who will be the proposer at the NEXT ethereum block.
   *          Should not be trusted when moving beyond the current epoch, since changes to the
   *          validator set might not be reflected when we actually reach that epoch (more changes
   *          might have happened).
   *
   * @dev     The proposer is selected from the validator set of the current epoch.
   *
   * @dev     Should only be access on-chain if epoch is setup, otherwise very expensive.
   *
   * @dev     A return value of address(0) means that the proposer is "open" and can be anyone.
   *
   * @dev     If the current epoch is the first epoch, returns address(0)
   *          If the current epoch is setup, we will return the proposer for the current slot
   *          If the current epoch is not setup, we will perform a sample as if it was (gas heavy)
   *
   * @return The address of the proposer
   */
  function getProposerAt(uint256 _ts) public view override(ILeonidas) returns (address) {
    uint256 epochNumber = getEpochAt(_ts);
    uint256 slot = getSlotAt(_ts);
    if (epochNumber == 0) {
      return address(0);
    }

    Epoch storage epoch = epochs[epochNumber];

    // If the epoch is setup, we can just return the proposer. Otherwise we have to emulate sampling
    if (epoch.sampleSeed != 0) {
      uint256 committeeSize = epoch.committee.length;
      if (committeeSize == 0) {
        return address(0);
      }

      return
        epoch.committee[_computeProposerIndex(epochNumber, slot, epoch.sampleSeed, committeeSize)];
    }

    // Allow anyone if there is no validator set
    if (validatorSet.length() == 0) {
      return address(0);
    }

    // Emulate a sampling of the validators
    uint256 sampleSeed = _getSampleSeed(epochNumber);
    address[] memory committee = _sampleValidators(sampleSeed);
    return committee[_computeProposerIndex(epochNumber, slot, sampleSeed, committee.length)];
  }

  /**
   * @notice  Adds a validator to the set WITHOUT setting up the epoch
   * @param _validator - The validator to add
   */
  function _addValidator(address _validator) internal {
    validatorSet.add(_validator);
  }

  /**
   * @notice  Propose a pending block from the point-of-view of sequencer selection. Will:
   *          - Setup the epoch if needed (if epoch committee is empty skips the rest)
   *          - Validate that the proposer is the proposer of the slot
   *          - Validate that the signatures for attestations are indeed from the validatorset
   *          - Validate that the number of valid attestations is sufficient
   *
   * @dev     Cases where errors are thrown:
   *          - If the epoch is not setup
   *          - If the proposer is not the real proposer AND the proposer is not open
   *          - If the number of valid attestations is insufficient
   *
   * @param _slot - The slot of the block
   * @param _signatures - The signatures of the committee members
   * @param _digest - The digest of the block
   */
  function _validateLeonidas(
    uint256 _slot,
    SignatureLib.Signature[] memory _signatures,
    bytes32 _digest,
    DataStructures.ExecutionFlags memory _flags
  ) internal view {
    uint256 ts = getTimestampForSlot(_slot);
    address proposer = getProposerAt(ts);

    // If the proposer is open, we allow anyone to propose without needing any signatures
    if (proposer == address(0)) {
      return;
    }

    // @todo We should allow to provide a signature instead of needing the proposer to broadcast.
    if (proposer != msg.sender) {
      revert Errors.Leonidas__InvalidProposer(proposer, msg.sender);
    }

    // @note  This is NOT the efficient way to do it, but it is a very convenient way for us to do it
    //        that allows us to reduce the number of code paths. Also when changed with optimistic for
    //        pleistarchus, this will be changed, so we can live with it.

    if (_flags.ignoreSignatures) {
      return;
    }

    address[] memory committee = getCommitteeAt(ts);

    uint256 needed = committee.length * 2 / 3 + 1;
    if (_signatures.length < needed) {
      revert Errors.Leonidas__InsufficientAttestationsProvided(needed, _signatures.length);
    }

    // Validate the attestations
    uint256 validAttestations = 0;

    bytes32 ethSignedDigest = _digest.toEthSignedMessageHash();

    for (uint256 i = 0; i < _signatures.length; i++) {
      SignatureLib.Signature memory signature = _signatures[i];
      if (signature.isEmpty) {
        continue;
      }

      // The verification will throw if invalid
      signature.verify(committee[i], ethSignedDigest);
      validAttestations++;
    }

    if (validAttestations < needed) {
      revert Errors.Leonidas__InsufficientAttestations(needed, validAttestations);
    }
  }

  /**
   * @notice  Samples a validator set for a specific epoch
   *
   * @dev     Only used internally, should never be called for anything but the "next" epoch
   *          Allowing us to always use `lastSeed`.
   *
   * @return The validators for the given epoch
   */
  function _sampleValidators(uint256 _seed) private view returns (address[] memory) {
    uint256 validatorSetSize = validatorSet.length();
    if (validatorSetSize == 0) {
      return new address[](0);
    }

    // If we have less validators than the target committee size, we just return the full set
    if (validatorSetSize <= TARGET_COMMITTEE_SIZE) {
      return validatorSet.values();
    }

    uint256[] memory indicies =
      SampleLib.computeCommitteeClever(TARGET_COMMITTEE_SIZE, validatorSetSize, _seed);

    address[] memory committee = new address[](TARGET_COMMITTEE_SIZE);
    for (uint256 i = 0; i < TARGET_COMMITTEE_SIZE; i++) {
      committee[i] = validatorSet.at(indicies[i]);
    }
    return committee;
  }

  /**
   * @notice  Get the sample seed for an epoch
   *
   * @dev     This should behave as walking past the line, but it does not currently do that.
   *          If there are entire skips, e.g., 1, 2, 5 and we then go back and try executing
   *          for 4 we will get an invalid value because we will read lastSeed which is from 5.
   *
   * @dev     The `_epoch` will never be 0 nor in the future
   *
   * @dev     The return value will be equal to keccak256(n, block.prevrandao) for n being the last epoch
   *          setup.
   *
   * @return The sample seed for the epoch
   */
  function _getSampleSeed(uint256 _epoch) private view returns (uint256) {
    if (_epoch == 0) {
      return type(uint256).max;
    }
    uint256 sampleSeed = epochs[_epoch].sampleSeed;
    if (sampleSeed != 0) {
      return sampleSeed;
    }

    sampleSeed = epochs[_epoch - 1].nextSeed;
    if (sampleSeed != 0) {
      return sampleSeed;
    }

    return lastSeed;
  }

  /**
   * @notice  Computes the epoch at a specific time
   *
   * @param _ts - The timestamp to compute the epoch for
   *
   * @return The computed epoch
   */
  function getEpochAt(uint256 _ts) public view returns (uint256) {
    return (_ts - GENESIS_TIME) / (EPOCH_DURATION * SLOT_DURATION);
  }

  /**
   * @notice  Computes the slot at a specific time
   *
   * @param _ts - The timestamp to compute the slot for
   *
   * @return The computed slot
   */
  function getSlotAt(uint256 _ts) public view returns (uint256) {
    return (_ts - GENESIS_TIME) / SLOT_DURATION;
  }

  /**
   * @notice  Computes the nextSeed for an epoch
   *
   * @dev     We include the `_epoch` instead of using the randao directly to avoid issues with foundry testing
   *          where randao == 0.
   *
   * @param _epoch - The epoch to compute the seed for
   *
   * @return The computed seed
   */
  function _computeNextSeed(uint256 _epoch) private view returns (uint256) {
    return uint256(keccak256(abi.encode(_epoch, block.prevrandao)));
  }

  /**
   * @notice  Computes the index of the committee member that acts as proposer for a given slot
   *
   * @param _epoch - The epoch to compute the proposer index for
   * @param _slot - The slot to compute the proposer index for
   * @param _seed - The seed to use for the computation
   * @param _size - The size of the committee
   *
   * @return The index of the proposer
   */
  function _computeProposerIndex(uint256 _epoch, uint256 _slot, uint256 _seed, uint256 _size)
    private
    pure
    returns (uint256)
  {
    return uint256(keccak256(abi.encode(_epoch, _slot, _seed))) % _size;
  }
}
