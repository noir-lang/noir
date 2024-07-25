// Verification Key Hash: 3b1c156f02c5934c94573e30a9d55a6398e8d1f616136797c008194d26892a55
// SPDX-License-Identifier: Apache-2.0
// Copyright 2022 Aztec
pragma solidity >=0.8.4;

library EcdsaUltraVerificationKey {
    function verificationKeyHash() internal pure returns (bytes32) {
        return 0x3b1c156f02c5934c94573e30a9d55a6398e8d1f616136797c008194d26892a55;
    }

    function loadVerificationKey(uint256 _vk, uint256 _omegaInverseLoc) internal pure {
        assembly {
            mstore(add(_vk, 0x00), 0x0000000000000000000000000000000000000000000000000000000000010000) // vk.circuit_size
            mstore(add(_vk, 0x20), 0x0000000000000000000000000000000000000000000000000000000000000006) // vk.num_inputs
            mstore(add(_vk, 0x40), 0x00eeb2cb5981ed45649abebde081dcff16c8601de4347e7dd1628ba2daac43b7) // vk.work_root
            mstore(add(_vk, 0x60), 0x30641e0e92bebef818268d663bcad6dbcfd6c0149170f6d7d350b1b1fa6c1001) // vk.domain_inverse
            mstore(add(_vk, 0x80), 0x149e839df8f43c6975d85aa1007d219354b3389f7c93c96935e531fe03d01f88) // vk.Q1.x
            mstore(add(_vk, 0xa0), 0x15af1728559ee0f81053b668fa9978c5fc81ee84d017bc955ccfa37c19bd42a0) // vk.Q1.y
            mstore(add(_vk, 0xc0), 0x29ba522730da8fa2a791946868afba96af78b025ba860d8e1d02e0325e677101) // vk.Q2.x
            mstore(add(_vk, 0xe0), 0x1434909cf7d729b2f4227d83569641d90c4a72d393390825de20cea7ddad8044) // vk.Q2.y
            mstore(add(_vk, 0x100), 0x03b90587c8746a60d96bc184e03c8469d813956caba6137040b350360357fe4f) // vk.Q3.x
            mstore(add(_vk, 0x120), 0x211f025196191d107ae492f80f0effeb1e9242069f333d405698365df4838d43) // vk.Q3.y
            mstore(add(_vk, 0x140), 0x0eae4a0952b07a5dbaf7750d79dae8fda3cfa4b5e7882413b6ada72c4297561e) // vk.Q4.x
            mstore(add(_vk, 0x160), 0x0fa2558fd5e0afe53d359b1ec584eb6c0fabad27e4909227d9a4457d588b2830) // vk.Q4.y
            mstore(add(_vk, 0x180), 0x01e7626aeb0ca204c26be5b01b3171994011b03f8966bb201303fc196c6c1a7e) // vk.Q_M.x
            mstore(add(_vk, 0x1a0), 0x07972ee3ae6e0a0cf4978b64cd08783f42c7ce9905f1fd35da4ff6fa0e1a18e2) // vk.Q_M.y
            mstore(add(_vk, 0x1c0), 0x03bd15837131c97d246c0aa57786e302b6d8227826104f70f56cba936a7b408e) // vk.Q_C.x
            mstore(add(_vk, 0x1e0), 0x1a2e3be55cd01c1a4f4ef33fa96986e37c56abc06876e7f7d76229fb9f122c4c) // vk.Q_C.y
            mstore(add(_vk, 0x200), 0x26d1d1578bb09f2f047035f103c3b32180c89b338e7d04ace8872b1154be6fb5) // vk.Q_ARITHMETIC.x
            mstore(add(_vk, 0x220), 0x10c4691982c731ec4e2bb8216e8af8405fbe96fe8fe305ef2c3e03444fe68f85) // vk.Q_ARITHMETIC.y
            mstore(add(_vk, 0x240), 0x1feb6cf63471a70e29caeee13eb393760c0f7d9e556327beb09a22b6b35e89f7) // vk.QSORT.x
            mstore(add(_vk, 0x260), 0x1a834941cde87aa7a82450b4f093f149df9937db2edbdab47fa7216fbcb33580) // vk.QSORT.y
            mstore(add(_vk, 0x280), 0x21245d6c0a4d2ff12b21a825f39f30e8f8cf9b259448d111183e975828539576) // vk.Q_ELLIPTIC.x
            mstore(add(_vk, 0x2a0), 0x16a409532c8a1693536e93b6ce9920bfc2e6796e8dfe404675a0cdf6ee77ee7a) // vk.Q_ELLIPTIC.y
            mstore(add(_vk, 0x2c0), 0x24005a1e8120ffcb3e5fc06ff50794b9d4b0bd70eabb1f8dfb342bec8a64dd61) // vk.Q_AUX.x
            mstore(add(_vk, 0x2e0), 0x2c18b52f602a5a9b4461872eff0712f56d128bb9364471f838d7b07f008660e3) // vk.Q_AUX.y
            mstore(add(_vk, 0x300), 0x02497b2d5e01266cea1f1bf4d9ad66e54045b3e388066db97b9623668728f65d) // vk.SIGMA1.x
            mstore(add(_vk, 0x320), 0x0156cae236ca46f64832b4b826804da6c7221ab5ca4cdadd53a1b787992307fe) // vk.SIGMA1.y
            mstore(add(_vk, 0x340), 0x2673cb9276dcc16be61e4c2ec24f6a881e771a273198ab0b392c26085a5f03b4) // vk.SIGMA2.x
            mstore(add(_vk, 0x360), 0x1384aef6995f8e632b76cce98d900e2535d92719be668a8f0e20c893c87f391a) // vk.SIGMA2.y
            mstore(add(_vk, 0x380), 0x11d48b7fc901d1e72489d937970ee3baea2662d268f9b1c08d71820a21ac6a39) // vk.SIGMA3.x
            mstore(add(_vk, 0x3a0), 0x126e543f1951015c8a56ff6d571e67da3cc52d2671f3ce8d258378edcfe8a8f5) // vk.SIGMA3.y
            mstore(add(_vk, 0x3c0), 0x0b41b102b59ecae092c04a4f09755db1dc4286c3072034ca23b7f885bcfec814) // vk.SIGMA4.x
            mstore(add(_vk, 0x3e0), 0x13bf888757f9fad73f21ab3a0ef53a286329dbf0aaaa935d1689d8554db05813) // vk.SIGMA4.y
            mstore(add(_vk, 0x400), 0x18f7cf965339d9c9d190296fa92f915767b0a8da455975f3e03fa98439fd7110) // vk.TABLE1.x
            mstore(add(_vk, 0x420), 0x0eecc02f9d44125407adbf00d56b086afd1adc5de536450afe05de382761b32f) // vk.TABLE1.y
            mstore(add(_vk, 0x440), 0x0bdfe662ea9f40f125ca5f7e99a8c6ba09b87ba8313864316745df862946c5c4) // vk.TABLE2.x
            mstore(add(_vk, 0x460), 0x0c5313c5b17634332920f54081fd46464a5ce9399e507c8fece9df28bff19033) // vk.TABLE2.y
            mstore(add(_vk, 0x480), 0x232ab86409f60c50fd5f04e879fbcbe60e358eb0337c5d0db1934277e1d8b1f2) // vk.TABLE3.x
            mstore(add(_vk, 0x4a0), 0x1fda66dfb58273345f2471dff55c51b6856241460272e64b4cc67cde65231e89) // vk.TABLE3.y
            mstore(add(_vk, 0x4c0), 0x024ccc0fcff3b515cdc97dde2fae5c516bf3c97207891801707142af02538a83) // vk.TABLE4.x
            mstore(add(_vk, 0x4e0), 0x27827250d02b7b67d084bfc52b26c722f33f75ae5098c109573bfe92b782e559) // vk.TABLE4.y
            mstore(add(_vk, 0x500), 0x302e6c8067a7ca14e1d75776754c1a3ad99d21056ae8e607ea66029cbe534906) // vk.TABLE_TYPE.x
            mstore(add(_vk, 0x520), 0x07f2eb44fd686bf54e604a6b40c9151b7123db580a23c064ef703af4013dbc2f) // vk.TABLE_TYPE.y
            mstore(add(_vk, 0x540), 0x00992a2f510c6371b9231c1d68d0e0fdbe10c5f4344de9441cc7c845afb37a1d) // vk.ID1.x
            mstore(add(_vk, 0x560), 0x13eb38f67d8c03245e6f0655f5d40c145b2c06dd1657d8da26dc75af0cefa0f7) // vk.ID1.y
            mstore(add(_vk, 0x580), 0x2ce905fbf9f932ae4f9b7b0feda15271b80921e9bf4e58c302ae99f1207fa4e7) // vk.ID2.x
            mstore(add(_vk, 0x5a0), 0x2c2a6dc03599757fc625b0e55984d3fb28a954d40eb54f988b52c55936076988) // vk.ID2.y
            mstore(add(_vk, 0x5c0), 0x0f547249b9aa5b9a951757893c059f8ed590366da4dd3ccd36aeac3069c7471f) // vk.ID3.x
            mstore(add(_vk, 0x5e0), 0x2be2746434bfe0ccb2390357b17f8ec70ff12fc3aad4500b8d1723ec6709a170) // vk.ID3.y
            mstore(add(_vk, 0x600), 0x19d1ed6b528ae5095d83167c3ba3578b36c7cd9249e47d10ceff352890d0938f) // vk.ID4.x
            mstore(add(_vk, 0x620), 0x1dcd2caa39e180a497ff98414548e5de682d19fc598b3cd44242f1bb53a0e078) // vk.ID4.y
            mstore(add(_vk, 0x640), 0x00) // vk.contains_recursive_proof
            mstore(add(_vk, 0x660), 0) // vk.recursive_proof_public_input_indices
            mstore(add(_vk, 0x680), 0x260e01b251f6f1c7e7ff4e580791dee8ea51d87a358e038b4efe30fac09383c1) // vk.g2_x.X.c1
            mstore(add(_vk, 0x6a0), 0x0118c4d5b837bcc2bc89b5b398b5974e9f5944073b32078b7e231fec938883b0) // vk.g2_x.X.c0
            mstore(add(_vk, 0x6c0), 0x04fc6369f7110fe3d25156c1bb9a72859cf2a04641f99ba4ee413c80da6a5fe4) // vk.g2_x.Y.c1
            mstore(add(_vk, 0x6e0), 0x22febda3c0c0632a56475b4214e5615e11e6dd3f96e6cea2854a87d4dacc5e55) // vk.g2_x.Y.c0
            mstore(_omegaInverseLoc, 0x0b5d56b77fe704e8e92338c0082f37e091126414c830e4c6922d5ac802d842d4) // vk.work_root_inverse
        }
    }
}
