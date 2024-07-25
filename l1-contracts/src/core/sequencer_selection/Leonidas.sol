// SPDX-License-Identifier: Apache-2.0
// Copyright 2024 Aztec Labs.
pragma solidity >=0.8.18;

import {Errors} from "../libraries/Errors.sol";
import {EnumerableSet} from "@oz/utils/structs/EnumerableSet.sol";
import {Ownable} from "@oz/access/Ownable.sol";
import {SignatureLib} from "./SignatureLib.sol";

import {ILeonidas} from "./ILeonidas.sol";

/**
 * @title   Leonidas
 * @author  Anaxandridas II
 * @notice  Leonidas is the spartan king, it is his job to select the warriors progressing the state of the kingdom.
 *          He define the structure needed for committee and leader selection and provides logic for validating that
 *          the block and its "evidence" follows his rules.
 *
 * @dev     Leonidas is depending on Ares to select warriors competently.
 *
 * @dev     Leonidas have one thing in mind, he provide a reference of the LOGIC going on for the spartan selection.
 *          He is not concerned about gas costs, he is a king, he just throw gas in the air like no-one cares.
 *          It will be the duty of his successor (Pleistarchus) to optimize the costs with same functionality.
 *
 */
contract Leonidas is Ownable, ILeonidas {
  using EnumerableSet for EnumerableSet.AddressSet;
  using SignatureLib for SignatureLib.Signature;

  /**
   * @notice  The structure of an epoch
   * @param committee - The validator set for the epoch
   * @param sampleSeed - The seed used to sample the validator set of the epoch
   * @param nextSeed - The seed used to influence the NEXT epoch
   */
  struct Epoch {
    address[] committee;
    uint256 sampleSeed;
    uint256 nextSeed;
  }

  // The size/duration of a slot in seconds, multiple of 12 to align with Ethereum blocks
  uint256 public constant SLOT_SIZE = 12 * 5;

  // The size/duration of an epoch in slots
  uint256 public constant EPOCH_SIZE = 32;

  // The target number of validators in a committee
  uint256 public constant TARGET_COMMITTEE_SIZE = EPOCH_SIZE;

  // The time that the contract was deployed
  uint256 public immutable GENESIS_TIME;

  // An enumerable set of validators that are up to date
  EnumerableSet.AddressSet private validatorSet;

  // A mapping to snapshots of the validator set
  mapping(uint256 epochNumber => Epoch epoch) public epochs;

  // The last stored randao value, same value as `seed` in the last inserted epoch
  uint256 internal lastSeed;

  constructor(address _ares) Ownable(_ares) {
    GENESIS_TIME = block.timestamp;

    // We will setup the initial epoch value
    uint256 seed = _computeNextSeed(0);
    epochs[0] = Epoch({committee: new address[](0), sampleSeed: type(uint256).max, nextSeed: seed});
    lastSeed = seed;
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
    validatorSet.add(_validator);
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

    // For epoch 0 the sampleSeed == type(uint256).max, so we will never enter this
    if (epoch.sampleSeed == 0) {
      epoch.sampleSeed = _getSampleSeed(epochNumber);
      epoch.nextSeed = lastSeed = _computeNextSeed(epochNumber);

      epoch.committee = _sampleValidators(epochNumber, epoch.sampleSeed);
    }
  }

  /**
   * @notice  Get the current epoch number
   *
   * @return The current epoch number
   */
  function getCurrentEpoch() public view override(ILeonidas) returns (uint256) {
    return (block.timestamp - GENESIS_TIME) / (EPOCH_SIZE * SLOT_SIZE);
  }

  /**
   * @notice  Get the current slot number
   *
   * @return The current slot number
   */
  function getCurrentSlot() public view override(ILeonidas) returns (uint256) {
    return (block.timestamp - GENESIS_TIME) / SLOT_SIZE;
  }

  /**
   * @notice  Get the proposer for the current slot
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
  function getCurrentProposer() public view override(ILeonidas) returns (address) {
    uint256 epochNumber = getCurrentEpoch();
    if (epochNumber == 0) {
      return address(0);
    }
    uint256 slot = getCurrentSlot();

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
    address[] memory committee = _sampleValidators(epochNumber, sampleSeed);
    return committee[_computeProposerIndex(epochNumber, slot, sampleSeed, committee.length)];
  }

  /**
   * @notice  Process a pending block from the point-of-view of sequencer selection. Will:
   *          - Setup the epoch if needed (if epoch committee is empty skips the rest)
   *          - Validate that the proposer is the current proposer
   *          - Validate that the signatures for attestations are indeed from the validatorset
   *          - Validate that the number of valid attestations is sufficient
   *
   * @dev     Cases where errors are thrown:
   *          - If the epoch is not setup
   *          - If the proposer is not the current proposer
   *          - If the number of valid attestations is insufficient
   */
  function _processPendingBlock(SignatureLib.Signature[] memory _signatures, bytes32 _digest)
    internal
  {
    setupEpoch();

    Epoch storage epoch = epochs[getCurrentEpoch()];

    // We should never enter this case because of `setupEpoch`
    if (epoch.sampleSeed == 0) {
      revert Errors.Leonidas__EpochNotSetup();
    }

    address proposer = getCurrentProposer();

    // If the proposer is open, we allow anyone to propose without needing any signatures
    if (proposer == address(0)) {
      return;
    }

    // @todo We should allow to provide a signature instead of needing the proposer to broadcast.
    if (proposer != msg.sender) {
      revert Errors.Leonidas__InvalidProposer(proposer, msg.sender);
    }

    // Validate the attestations
    uint256 validAttestations = 0;
    for (uint256 i = 0; i < _signatures.length; i++) {
      SignatureLib.Signature memory signature = _signatures[i];
      if (signature.isEmpty) {
        continue;
      }

      // The verification will throw if invalid
      signature.verify(epoch.committee[i], _digest);
      validAttestations++;
    }
    uint256 needed = epoch.committee.length * 2 / 3 + 1;
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
   * @dev     The first epoch will always return an empty list
   *          If the validator set is empty, we return an empty list
   *          If the validator set is smaller than the target committee size, we return the full set
   *          If the validator set is larger than the target committee size, we sample the validators
   *          by using the seed of the previous epoch to compute an offset for the validator set and then
   *          we take the next `TARGET_COMMITTEE_SIZE` validators from that offset (wrapping around).
   *
   * @param _epoch - The epoch to sample the validators for
   *
   * @return The validators for the given epoch
   */
  function _sampleValidators(uint256 _epoch, uint256 _seed) private view returns (address[] memory) {
    // If we are in the first epoch, we just return an empty list
    if (_epoch == 0) {
      return new address[](0);
    }

    uint256 validatorSetSize = validatorSet.length();
    if (validatorSetSize == 0) {
      return new address[](0);
    }

    // If we have less validators than the target committee size, we just return the full set
    if (validatorSet.length() <= TARGET_COMMITTEE_SIZE) {
      return validatorSet.values();
    }

    // @todo Issue(#7603): The sampling should be improved

    uint256 offset = _seed % validatorSetSize;
    address[] memory validators = new address[](TARGET_COMMITTEE_SIZE);
    for (uint256 i = 0; i < TARGET_COMMITTEE_SIZE; i++) {
      validators[i] = validatorSet.at((offset + i) % validatorSetSize);
    }
    return validators;
  }

  /**
   * @notice  Get the sample seed for an epoch
   *
   * @dev     The `_epoch` will never be 0 nor in the future
   *
   * @dev     The return value will be equal to keccak256(n, block.prevrandao) for n being the last epoch
   *          setup.
   *
   * @return The sample seed for the epoch
   */
  function _getSampleSeed(uint256 _epoch) private view returns (uint256) {
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
