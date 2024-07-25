pragma solidity ^0.8.13;

import {Errors} from "../libraries/Errors.sol";

library SignatureLib {
  struct Signature {
    bool isEmpty;
    uint8 v;
    bytes32 r;
    bytes32 s;
  }

  /**
   * @notice Verified a signature, throws if the signature is invalid or empty
   *
   * @param _signature - The signature to verify
   * @param _signer - The expected signer of the signature
   * @param _digest - The digest that was signed
   */
  function verify(Signature memory _signature, address _signer, bytes32 _digest) internal pure {
    if (_signature.isEmpty) {
      revert Errors.SignatureLib__CannotVerifyEmpty();
    }
    address recovered = ecrecover(_digest, _signature.v, _signature.r, _signature.s);
    if (_signer != recovered) {
      revert Errors.SignatureLib__InvalidSignature(_signer, recovered);
    }
  }
}
