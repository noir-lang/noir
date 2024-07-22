// Verification Key Hash: f7bbd1b4758c8616f966f56728b3d7127a0d1ca6763cbaf70b4719914be476bd
// SPDX-License-Identifier: Apache-2.0
// Copyright 2022 Aztec
pragma solidity >=0.8.4;

library Add2UltraVerificationKey {
    function verificationKeyHash() internal pure returns (bytes32) {
        return 0xf7bbd1b4758c8616f966f56728b3d7127a0d1ca6763cbaf70b4719914be476bd;
    }

    function loadVerificationKey(uint256 _vk, uint256 _omegaInverseLoc) internal pure {
        assembly {
            mstore(add(_vk, 0x00), 0x0000000000000000000000000000000000000000000000000000000000000010) // vk.circuit_size
            mstore(add(_vk, 0x20), 0x0000000000000000000000000000000000000000000000000000000000000003) // vk.num_inputs
            mstore(add(_vk, 0x40), 0x21082ca216cbbf4e1c6e4f4594dd508c996dfbe1174efb98b11509c6e306460b) // vk.work_root
            mstore(add(_vk, 0x60), 0x2d5e098bb31e86271ccb415b196942d755b0a9c3f21dd9882fa3d63ab1000001) // vk.domain_inverse
            mstore(add(_vk, 0x80), 0x0fe8527a8494f827e4b332060e5569bdfe47aadc475cb1f03b4d222b0804e463) // vk.Q1.x
            mstore(add(_vk, 0xa0), 0x2d051f6d8a9eec7ae2622b4b259fb91d942a8892632cffcdad4c03e2c9081593) // vk.Q1.y
            mstore(add(_vk, 0xc0), 0x21be04985f898ef5e2a80f32fbaa16cd3eb8c451c243db46144b97113c8e556a) // vk.Q2.x
            mstore(add(_vk, 0xe0), 0x2913bcdde62d6d2143aa6a0fed511ca527994df2cba6a780baa29dddac161de9) // vk.Q2.y
            mstore(add(_vk, 0x100), 0x15bf0818d578953a624a127aa04d4b0aa1b09d3fe69b9a29e2546a41e9b08049) // vk.Q3.x
            mstore(add(_vk, 0x120), 0x0148f0d2abf2ca1fe6cc20fdef0a8d4a88eb9602eaf48f0c1c02445c27cb9592) // vk.Q3.y
            mstore(add(_vk, 0x140), 0x02d6fd9e84dbe74b7531e1801405a1c292117b1a17fefe9de0bfd9edf1a84bf9) // vk.Q4.x
            mstore(add(_vk, 0x160), 0x293c6ab3c06a0669af13393a82c60a459a3b2a0b768da45ac7af7f2aec40fc42) // vk.Q4.y
            mstore(add(_vk, 0x180), 0x2950076760523510abcfe90fa550b964e84b338f73af5222cdbbaefdacd4484e) // vk.Q_M.x
            mstore(add(_vk, 0x1a0), 0x2e4e3e272c7b78ad894559812d7766e05615a8f7050a43d7ed1367adf30a9319) // vk.Q_M.y
            mstore(add(_vk, 0x1c0), 0x1798c37010a4285e1774c1ad35779886380ee5ceee0ba183927e2a2103301a68) // vk.Q_C.x
            mstore(add(_vk, 0x1e0), 0x2935f9e4d47a8e39aa0107f31a84584b47d903cfeb9690f6d850dc8ea7d2f4ea) // vk.Q_C.y
            mstore(add(_vk, 0x200), 0x205aff7186f2cedc99e9cadb7930ef25f296991e5a7b574a33d6249f0a5db1f9) // vk.Q_ARITHMETIC.x
            mstore(add(_vk, 0x220), 0x0beadc757bee3a2be1f6425385547099fc1eeeb1fed5a8552b07e997251d7c45) // vk.Q_ARITHMETIC.y
            mstore(add(_vk, 0x240), 0x2cbce7beee3076b78dace04943d69d0d9e28aa6d00e046852781a5f20816645c) // vk.QSORT.x
            mstore(add(_vk, 0x260), 0x2bc27ec2e1612ea284b08bcc55b6f2fd915d11bfedbdc0e59de09e5b28952080) // vk.QSORT.y
            mstore(add(_vk, 0x280), 0x0ad34b5e8db72a5acf4427546c7294be6ed4f4d252a79059e505f9abc1bdf3ed) // vk.Q_ELLIPTIC.x
            mstore(add(_vk, 0x2a0), 0x1e5b26790a26eb340217dd9ad28dbf90a049f42a3852acd45e6f521f24b4900e) // vk.Q_ELLIPTIC.y
            mstore(add(_vk, 0x2c0), 0x155a0f51fec78c33ffceb7364d69d7ac27e570ae50bc180509764eb3fef94815) // vk.Q_AUX.x
            mstore(add(_vk, 0x2e0), 0x1c1c4720bed44a591d97cbc72b6e44b644999713a8d3c66e9054aa5726324c76) // vk.Q_AUX.y
            mstore(add(_vk, 0x300), 0x01d8b8ff3b1674e57a7d30ce1d9e07c686174c643eb20d38e604eec7095248a9) // vk.SIGMA1.x
            mstore(add(_vk, 0x320), 0x261015d69327a58810e6eb1052ed694914b7a89034e1334c50b9e70a161489b7) // vk.SIGMA1.y
            mstore(add(_vk, 0x340), 0x1987df111730a8a6a650423757dbf048f3f43860a7d24a5e2e8bd67b6931ca67) // vk.SIGMA2.x
            mstore(add(_vk, 0x360), 0x1fe074ff24b35d7830bec2a3fad7e53038ffb5e2a3f718a23660d68daffc2953) // vk.SIGMA2.y
            mstore(add(_vk, 0x380), 0x1995a6246a4cf48e66d1f4753d001b67712f57a94b364498c12b7048f162665f) // vk.SIGMA3.x
            mstore(add(_vk, 0x3a0), 0x0b09e4f696f8fcdae85a72b71cc2bc2be2be2960d4fa0078d79c32d21619bd5e) // vk.SIGMA3.y
            mstore(add(_vk, 0x3c0), 0x267b628499addfdfc2ff596957b0aa713baa1054a78cd80dcb0b588bfa209adf) // vk.SIGMA4.x
            mstore(add(_vk, 0x3e0), 0x119db2893b20259b419e53e6686f2fba9934c9e1af0ffa2ede920718eea14086) // vk.SIGMA4.y
            mstore(add(_vk, 0x400), 0x02c397073c8abce6d4140c9b961209dd783bff1a1cfc999bb29859cfb16c46fc) // vk.TABLE1.x
            mstore(add(_vk, 0x420), 0x2b7bba2d1efffce0d033f596b4d030750599be670db593af86e1923fe8a1bb18) // vk.TABLE1.y
            mstore(add(_vk, 0x440), 0x2c71c58b66498f903b3bbbda3d05ce8ffb571a4b3cf83533f3f71b99a04f6e6b) // vk.TABLE2.x
            mstore(add(_vk, 0x460), 0x039dce37f94d1bbd97ccea32a224fe2afaefbcbd080c84dcea90b54f4e0a858f) // vk.TABLE2.y
            mstore(add(_vk, 0x480), 0x27dc44977efe6b3746a290706f4f7275783c73cfe56847d848fd93b63bf32083) // vk.TABLE3.x
            mstore(add(_vk, 0x4a0), 0x0a5366266dd7b71a10b356030226a2de0cbf2edc8f085b16d73652b15eced8f5) // vk.TABLE3.y
            mstore(add(_vk, 0x4c0), 0x136097d79e1b0ae373255e8760c49900a7588ec4d6809c90bb451005a3de3077) // vk.TABLE4.x
            mstore(add(_vk, 0x4e0), 0x13dd7515ccac4095302d204f06f0bff2595d77bdf72e4acdb0b0b43969860d98) // vk.TABLE4.y
            mstore(add(_vk, 0x500), 0x16ff3501369121d410b445929239ba057fe211dad1b706e49a3b55920fac20ec) // vk.TABLE_TYPE.x
            mstore(add(_vk, 0x520), 0x1e190987ebd9cf480f608b82134a00eb8007673c1ed10b834a695adf0068522a) // vk.TABLE_TYPE.y
            mstore(add(_vk, 0x540), 0x1e44194e60f0ab4ee0f77adc50f4220944f94301aa6da3016a226de04de52f4c) // vk.ID1.x
            mstore(add(_vk, 0x560), 0x2a017d0d9f40d0aeb5c8152ffddec56c2c7bea37dfbd20be6bed19efd743397a) // vk.ID1.y
            mstore(add(_vk, 0x580), 0x0868357b28039385c5a5058b6d358ebb29f26f9890d6cc6401f4921d5884edca) // vk.ID2.x
            mstore(add(_vk, 0x5a0), 0x1060afe929554ca473103f5e68193c36fb6e229dde8edf7ec858b12d7e8be485) // vk.ID2.y
            mstore(add(_vk, 0x5c0), 0x1953e657c2c941f0eb52e94a96b64a0152c7cc3baff59a345fd6ecd311f313af) // vk.ID3.x
            mstore(add(_vk, 0x5e0), 0x08579320bf2aa71698b64152f1f3f4afe1873af96d6403dcaf56908dc78c6704) // vk.ID3.y
            mstore(add(_vk, 0x600), 0x2eea648c8732596b1314fe2a4d2f05363f0c994e91cecad25835338edee2294f) // vk.ID4.x
            mstore(add(_vk, 0x620), 0x0ab49886c2b94bd0bd3f6ed1dbbe2cb2671d2ae51d31c1210433c3972bb64578) // vk.ID4.y
            mstore(add(_vk, 0x640), 0x00) // vk.contains_recursive_proof
            mstore(add(_vk, 0x660), 0) // vk.recursive_proof_public_input_indices
            mstore(add(_vk, 0x680), 0x260e01b251f6f1c7e7ff4e580791dee8ea51d87a358e038b4efe30fac09383c1) // vk.g2_x.X.c1
            mstore(add(_vk, 0x6a0), 0x0118c4d5b837bcc2bc89b5b398b5974e9f5944073b32078b7e231fec938883b0) // vk.g2_x.X.c0
            mstore(add(_vk, 0x6c0), 0x04fc6369f7110fe3d25156c1bb9a72859cf2a04641f99ba4ee413c80da6a5fe4) // vk.g2_x.Y.c1
            mstore(add(_vk, 0x6e0), 0x22febda3c0c0632a56475b4214e5615e11e6dd3f96e6cea2854a87d4dacc5e55) // vk.g2_x.Y.c0
            mstore(_omegaInverseLoc, 0x02e40daf409556c02bfc85eb303402b774954d30aeb0337eb85a71e6373428de) // vk.work_root_inverse
        }
    }
}
