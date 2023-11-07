// Verification Key Hash: e22392a1f3bddc5d1f000fd4920b5c0e7f8282ee348c68538ea51321fda78a6f
// SPDX-License-Identifier: Apache-2.0
// Copyright 2022 Aztec
pragma solidity >=0.8.4;

library RecursiveUltraVerificationKey {
    function verificationKeyHash() internal pure returns (bytes32) {
        return 0xe22392a1f3bddc5d1f000fd4920b5c0e7f8282ee348c68538ea51321fda78a6f;
    }

    function loadVerificationKey(uint256 _vk, uint256 _omegaInverseLoc) internal pure {
        assembly {
            mstore(add(_vk, 0x00), 0x0000000000000000000000000000000000000000000000000000000000040000) // vk.circuit_size
            mstore(add(_vk, 0x20), 0x0000000000000000000000000000000000000000000000000000000000000010) // vk.num_inputs
            mstore(add(_vk, 0x40), 0x19ddbcaf3a8d46c15c0176fbb5b95e4dc57088ff13f4d1bd84c6bfa57dcdc0e0) // vk.work_root
            mstore(add(_vk, 0x60), 0x30644259cd94e7dd5045d7a27013b7fcd21c9e3b7fa75222e7bda49b729b0401) // vk.domain_inverse
            mstore(add(_vk, 0x80), 0x1955384fa963070c967ff2c960277b4e296aae8d72c72e01930356b9e66e82f0) // vk.Q1.x
            mstore(add(_vk, 0xa0), 0x1abfdcc530bde7617e8bdacdb27582d2c313b5ac663a9d47075ccd7ea20be189) // vk.Q1.y
            mstore(add(_vk, 0xc0), 0x019e10458c6caa9d5be700d9b289766239bef8ef7f9608300f75e4db84014d7b) // vk.Q2.x
            mstore(add(_vk, 0xe0), 0x2c8b6fc311eb5850c154e438fc9ffba848c332d3b1f517266751cff56f711890) // vk.Q2.y
            mstore(add(_vk, 0x100), 0x16087fc135eeed06f55acc3a59da6eff09599c013fbc397e742e7e7c3e34529a) // vk.Q3.x
            mstore(add(_vk, 0x120), 0x025586359ebb3a81602ac1266da2503f320d26b9b18a3835a75f5ea587363d9b) // vk.Q3.y
            mstore(add(_vk, 0x140), 0x049008f625581a490bb8fcf4ea3c648c6fd4d802cdffe7866175efd7b5664185) // vk.Q4.x
            mstore(add(_vk, 0x160), 0x25e38e7a8cda67d926bdf8db7ebdd535499f4354f1e902ef27aeb79c63d2c233) // vk.Q4.y
            mstore(add(_vk, 0x180), 0x113e080e177eeabec67380d0d0ecfdbfd9a8f7cc9e02c8d8445c328abbfeb9a5) // vk.Q_M.x
            mstore(add(_vk, 0x1a0), 0x10f95dae13bfe0c0a3efbe10855b52c43269d8aa611525dbbcb1f1d0eb42f848) // vk.Q_M.y
            mstore(add(_vk, 0x1c0), 0x23f14dc05ec047d53df22710cdfd3cb4a44963811078a600811aa06d9576b4c0) // vk.Q_C.x
            mstore(add(_vk, 0x1e0), 0x0821bba87eb570b4f41b432624bb2bf013a9b129e7bc0c0178bcc2adc1c47606) // vk.Q_C.y
            mstore(add(_vk, 0x200), 0x26486dece09dab5a8e4e757625088433f1d8123e8fda3693d4a7993f621f1eed) // vk.Q_ARITHMETIC.x
            mstore(add(_vk, 0x220), 0x286019a7e6055aef52b91c449a1c2b9abbcb92595118160efc96ced10ac4b6e4) // vk.Q_ARITHMETIC.y
            mstore(add(_vk, 0x240), 0x2ca5a08c8d2cc428aa539aab26c0ae71d28ed89e61fff9ef5c9eb896748e01c0) // vk.QSORT.x
            mstore(add(_vk, 0x260), 0x124d0e50734e64db09937d992e57c88c3d82f17786ee7691191da883af81f7cb) // vk.QSORT.y
            mstore(add(_vk, 0x280), 0x1ade93a940dab58eb305c26f147e387aa2ce033cd98b3f6d92d440a7ec159d7e) // vk.Q_ELLIPTIC.x
            mstore(add(_vk, 0x2a0), 0x0bfe74216774dc130b6219ffd3ca3d716dde56532ef454e002b5e7cc1a714f06) // vk.Q_ELLIPTIC.y
            mstore(add(_vk, 0x2c0), 0x11aa3c2e6abd71b46496cc7258ffb26e454779dd7a861c9b170df7b6d19866bb) // vk.Q_AUX.x
            mstore(add(_vk, 0x2e0), 0x078a7416251f2354b81f7f23674a442733beaa73928c52482e09af67e4266630) // vk.Q_AUX.y
            mstore(add(_vk, 0x300), 0x18b0d041e64959a1b4c8aec2988ed0781a8e71e3b399e9e6f1519553e7d4b844) // vk.SIGMA1.x
            mstore(add(_vk, 0x320), 0x051faec8bc66561eb3dd53d7e9f062a69726ee92df29f337c1152b4838a6ecb5) // vk.SIGMA1.y
            mstore(add(_vk, 0x340), 0x1b3db046a836a946d73153637681ae12a3747e76100a973ba3e57a60bf05f8b8) // vk.SIGMA2.x
            mstore(add(_vk, 0x360), 0x2a0d1cb0659525e3e515020d4728b9deb1aac70c1286eb565e2589da5700caac) // vk.SIGMA2.y
            mstore(add(_vk, 0x380), 0x2da060ec79d4280499b69fde005a3712a9b694118f9332af6e1611659ea05d10) // vk.SIGMA3.x
            mstore(add(_vk, 0x3a0), 0x0de556d12e70d90ed705a5542c16a55a44910532e1f24c2252649f0b061af019) // vk.SIGMA3.y
            mstore(add(_vk, 0x3c0), 0x101702660aecee7905d290afb56978ff8756662cb0589bf1260b7aa0feb8e044) // vk.SIGMA4.x
            mstore(add(_vk, 0x3e0), 0x29d27af726556b6b97d2998f0ed57fecbb2fa27cefc51fee5a96a4ada07c0d2d) // vk.SIGMA4.y
            mstore(add(_vk, 0x400), 0x09796190fd3ba909c6530c89811df9b5b4f5f2fe6501ec21dd864b20673fc02c) // vk.TABLE1.x
            mstore(add(_vk, 0x420), 0x00b9c2423e310caa43e1eb83b55f53977fccbed85422df8935635d77d146bf39) // vk.TABLE1.y
            mstore(add(_vk, 0x440), 0x217dad26ccc0c543ec5750513e9365a5cae8164b08d364efcf4b5890ff05f334) // vk.TABLE2.x
            mstore(add(_vk, 0x460), 0x1db28433f6bde424423f3587787f81c48101d2dc6e54b431332cb275f8518c62) // vk.TABLE2.y
            mstore(add(_vk, 0x480), 0x2cc2d90f2da7f4ec16b7fe61babd4fb9b580ecff03c471764dd67a8c433afab5) // vk.TABLE3.x
            mstore(add(_vk, 0x4a0), 0x3032b9ff096a43ce326cc63ffc6a86dcb913fb1f7700939f5304f6c6beb24574) // vk.TABLE3.y
            mstore(add(_vk, 0x4c0), 0x1f4c58502ca713ed0bffb4ff31ed55e557e83a37d31b8e703aa9219d6158e2d2) // vk.TABLE4.x
            mstore(add(_vk, 0x4e0), 0x0b0d5ed5432c5e7b56344c1d26ce0d9f632e8f8aa52505d6c89f6da89f357fa8) // vk.TABLE4.y
            mstore(add(_vk, 0x500), 0x1ec56cfb03ca703e6c5b12bc25735a6277ba8e195789f871273e0ab6108c69dc) // vk.TABLE_TYPE.x
            mstore(add(_vk, 0x520), 0x15dd65957a13632642739159f99cb0bc793a3e9fd3317b11de1887305b1f0ba0) // vk.TABLE_TYPE.y
            mstore(add(_vk, 0x540), 0x1132be2c2fba72cd6196c25c3f9e4a2607225e1dd9b1df156278bc70eaef9833) // vk.ID1.x
            mstore(add(_vk, 0x560), 0x15ac1cf6f0d69580d63e3cef0fbe784daddf5d89cd0532e6c3bac8110f713739) // vk.ID1.y
            mstore(add(_vk, 0x580), 0x0a54083d0241492b018ff651dd4a94f694ac29815be96788bbcc1f6731da2c2f) // vk.ID2.x
            mstore(add(_vk, 0x5a0), 0x1ac60ad10f90e6a3dae7fc97aa1e86be376a8e477e320ce1a2c5c667587414b0) // vk.ID2.y
            mstore(add(_vk, 0x5c0), 0x1499c048e87fea28057e1ec5c3a23c11556bdc658626b07edfedf2739ef2eae2) // vk.ID3.x
            mstore(add(_vk, 0x5e0), 0x280fec47ce5e775c01ebfb880fed2fa60ae7c7434d5de1d54dfe3f66cf7e1186) // vk.ID3.y
            mstore(add(_vk, 0x600), 0x2daba5e0c0a440cc134049635e97d3f4fdbe0709c73f0480fad500f51542c5ae) // vk.ID4.x
            mstore(add(_vk, 0x620), 0x064089effdda7af9ea5de9407b5e96f826001c7d7cef054877ff0203e7ad229e) // vk.ID4.y
            mstore(add(_vk, 0x640), 0x01) // vk.contains_recursive_proof
            mstore(add(_vk, 0x660), 0) // vk.recursive_proof_public_input_indices
            mstore(add(_vk, 0x680), 0x260e01b251f6f1c7e7ff4e580791dee8ea51d87a358e038b4efe30fac09383c1) // vk.g2_x.X.c1
            mstore(add(_vk, 0x6a0), 0x0118c4d5b837bcc2bc89b5b398b5974e9f5944073b32078b7e231fec938883b0) // vk.g2_x.X.c0
            mstore(add(_vk, 0x6c0), 0x04fc6369f7110fe3d25156c1bb9a72859cf2a04641f99ba4ee413c80da6a5fe4) // vk.g2_x.Y.c1
            mstore(add(_vk, 0x6e0), 0x22febda3c0c0632a56475b4214e5615e11e6dd3f96e6cea2854a87d4dacc5e55) // vk.g2_x.Y.c0
            mstore(_omegaInverseLoc, 0x036853f083780e87f8d7c71d111119c57dbe118c22d5ad707a82317466c5174c) // vk.work_root_inverse
        }
    }
}
