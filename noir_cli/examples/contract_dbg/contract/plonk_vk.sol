
// SPDX-License-Identifier: GPL-2.0-only
// Copyright 2020 Spilsbury Holdings Ltd

pragma solidity >=0.6.0 <0.7.0;
pragma experimental ABIEncoderV2;

import {Types} from '../cryptography/Types.sol';
import {PairingsBn254} from '../cryptography/PairingsBn254.sol';

library PLONK_VK {
  using PairingsBn254 for Types.G1Point;
  using PairingsBn254 for Types.G2Point;
  using PairingsBn254 for Types.Fr;

  function get_verification_key() internal pure returns (Types.VerificationKey memory) {
    Types.VerificationKey memory vk;

    vk.circuit_size = 16;
    vk.num_inputs = 0;
    vk.work_root = PairingsBn254.new_fr(
      0x21082ca216cbbf4e1c6e4f4594dd508c996dfbe1174efb98b11509c6e306460b
    );
    vk.domain_inverse = PairingsBn254.new_fr(
      0x2d5e098bb31e86271ccb415b196942d755b0a9c3f21dd9882fa3d63ab1000001
    );
    vk.work_root_inverse = PairingsBn254.new_fr(
      0x02e40daf409556c02bfc85eb303402b774954d30aeb0337eb85a71e6373428de
    );
    vk.Q1 = PairingsBn254.new_g1(
      0x0285dc07f90275978b7e63a3daa3730ecbc0cf449e2e9bda0ef46b3d6197e33e,
      0x0cd7b917bcca649eb13dc023e03d4a01b6d89ff57ca7abdc90b5ab548f890a31
    );
    vk.Q2 = PairingsBn254.new_g1(
      0x15638a5d106842dd78f4b5c452d23ed1061c562bba63e84bd7d6135c0bfe5078,
      0x13e926e3c71f87d96fd59f2077ededec393f2847621d4716182da60b8e913c33
    );
    vk.Q3 = PairingsBn254.new_g1(
      0x0b2f8e59c28b02e5e2279a4dcc2d009693a37eb93cb55abcea461bdfc5d89587,
      0x138e7805ab21d8bfe79a908b4410531c9306b7738b24df6c2d991de2823d6e1b
    );
    vk.Q4 = PairingsBn254.new_g1(
      0x2950076760523510abcfe90fa550b964e84b338f73af5222cdbbaefdacd4484e,
      0x2e4e3e272c7b78ad894559812d7766e05615a8f7050a43d7ed1367adf30a9319
    );
    vk.Q5 = PairingsBn254.new_g1(
      0x2950076760523510abcfe90fa550b964e84b338f73af5222cdbbaefdacd4484e,
      0x2e4e3e272c7b78ad894559812d7766e05615a8f7050a43d7ed1367adf30a9319
    );
    vk.QM = PairingsBn254.new_g1(
      0x1e8910796aa673d1dd992b14368ceed5565c973219aa1655cd56251ec9205a4e,
      0x03ed97fb3f96d860541bb5167ebead52ccb34ab921cbdcf10ac337b2fdcd3d6b
    );
    vk.QC = PairingsBn254.new_g1(
      0x1b58f0bdde0575d8a03d5f0357e236e6cd048073e87233b8580c3c357352597d,
      0x039ae2dc36fdc8dfbbfc1112e7bf9ebb994484fa2c29b4816cd67686c3c326d4
    );
    vk.QARITH = PairingsBn254.new_g1(
      0x01d31820b3233ab6096d57791d252d4e6f58426151f3b3fbb9887e4b98cd143d,
      0x03bc54189c0b12de789da9df23e618e2d25a0fd7849130a72df7c6980a415b5c
    );
    vk.QECC = PairingsBn254.new_g1(
      0x2950076760523510abcfe90fa550b964e84b338f73af5222cdbbaefdacd4484e,
      0x2e4e3e272c7b78ad894559812d7766e05615a8f7050a43d7ed1367adf30a9319
    );
    vk.QRANGE = PairingsBn254.new_g1(
      0x2950076760523510abcfe90fa550b964e84b338f73af5222cdbbaefdacd4484e,
      0x2e4e3e272c7b78ad894559812d7766e05615a8f7050a43d7ed1367adf30a9319
    );
    vk.QLOGIC = PairingsBn254.new_g1(
      0x2950076760523510abcfe90fa550b964e84b338f73af5222cdbbaefdacd4484e,
      0x2e4e3e272c7b78ad894559812d7766e05615a8f7050a43d7ed1367adf30a9319
    );
    vk.sigma_commitments[0] = PairingsBn254.new_g1(
      0x23b8560bd0836e2ed2440cf9adc3b1c0d7c45030481f4624b241e13fa3437030,
      0x01d9d31d3971dd82fe97003a5661771bf43deaff0de60b47e8e0cf300fe2256d
    );
    vk.sigma_commitments[1] = PairingsBn254.new_g1(
      0x0a800a0eaa4716d2a2c2f962f9079ff58319bfc57c15f908926a05450f1550be,
      0x2f409bb8abba0aa4927287a689099b92dc347b4a22269e063207316e0b47e70e
    );
    vk.sigma_commitments[2] = PairingsBn254.new_g1(
      0x15afebd8b6da70d526febe5177b3c71d78f99dc29f5f5dfab7fb856602604e76,
      0x1589d7dbdc1578c045902c63d6c0bfcd40cd4f997ac4d07ca10541e0163f781f
    );
    vk.sigma_commitments[3] = PairingsBn254.new_g1(
      0x2f43de15cdf68b9b3338639e4f5a9dc5dde5612c48bf8f44cff4b8c09c95a234,
      0x1df6b325275d25e70cf6e8fa3b627303972c96c16e4b5e03cd0cabdea6fc7e1a
    );
    vk.permutation_non_residues[0] = PairingsBn254.new_fr(
      0x0000000000000000000000000000000000000000000000000000000000000005
    );
    vk.permutation_non_residues[1] = PairingsBn254.new_fr(
      0x0000000000000000000000000000000000000000000000000000000000000006
    );
    vk.permutation_non_residues[2] = PairingsBn254.new_fr(
      0x0000000000000000000000000000000000000000000000000000000000000007
    );
    vk.g2_x = PairingsBn254.new_g2([
      0x260e01b251f6f1c7e7ff4e580791dee8ea51d87a358e038b4efe30fac09383c1,
      0x0118c4d5b837bcc2bc89b5b398b5974e9f5944073b32078b7e231fec938883b0
    ],[
      0x04fc6369f7110fe3d25156c1bb9a72859cf2a04641f99ba4ee413c80da6a5fe4,
      0x22febda3c0c0632a56475b4214e5615e11e6dd3f96e6cea2854a87d4dacc5e55
    ]);
    return vk;
  }
}
