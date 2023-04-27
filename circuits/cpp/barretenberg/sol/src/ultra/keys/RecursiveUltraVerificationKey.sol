// Verification Key Hash: 507de35addf16b79526d713259492d5d1764fdb6ce55ff4ccb03c147b72f381a
// SPDX-License-Identifier: Apache-2.0
// Copyright 2022 Aztec
pragma solidity >=0.8.4;

library RecursiveUltraVerificationKey {
    function verificationKeyHash() internal pure returns (bytes32) {
        return 0x507de35addf16b79526d713259492d5d1764fdb6ce55ff4ccb03c147b72f381a;
    }

    function loadVerificationKey(uint256 _vk, uint256 _omegaInverseLoc) internal pure {
        assembly {
            mstore(add(_vk, 0x00), 0x0000000000000000000000000000000000000000000000000000000000080000) // vk.circuit_size
            mstore(add(_vk, 0x20), 0x0000000000000000000000000000000000000000000000000000000000000010) // vk.num_inputs
            mstore(add(_vk, 0x40), 0x2260e724844bca5251829353968e4915305258418357473a5c1d597f613f6cbd) // vk.work_root
            mstore(add(_vk, 0x60), 0x3064486657634403844b0eac78ca882cfd284341fcb0615a15cfcd17b14d8201) // vk.domain_inverse
            mstore(add(_vk, 0x80), 0x18fe72968b540c1dad6c7648fcb3407edfc489d8dcf3fdce314c1f0e72684c43) // vk.Q1.x
            mstore(add(_vk, 0xa0), 0x16f49263ee016852edfed2e84bf44c22b31064b9034b62059329b2af2f349c37) // vk.Q1.y
            mstore(add(_vk, 0xc0), 0x1c382676d0f8e5691def3a60d533850f573c36aa200ab364c091acc4a7eb094f) // vk.Q2.x
            mstore(add(_vk, 0xe0), 0x17c05ca7ea679681a3cf772fabcf2c1a988e39910f1ba8de3d1f68ffb0effda1) // vk.Q2.y
            mstore(add(_vk, 0x100), 0x257d75dead2d8cbb2f63b3592a762a2c2dbe0195a533736fd01982370e768676) // vk.Q3.x
            mstore(add(_vk, 0x120), 0x258b6d74446f5e532bce6e1a62372a82986eac9801c13a8553f373c30398a47c) // vk.Q3.y
            mstore(add(_vk, 0x140), 0x290ff6a808f6abe7508a8c884ea0fc2f819e23a5b6d7c2dd1105da2a3f0637e0) // vk.Q4.x
            mstore(add(_vk, 0x160), 0x2e6c3c419be44ed56b61069a06e980360f58830ad52b38bb69de92c456ebf0ca) // vk.Q4.y
            mstore(add(_vk, 0x180), 0x282e6e14bbedfc7ef013feb4877ce9098389abfd3ad8899c957be4fdb20d0454) // vk.Q_M.x
            mstore(add(_vk, 0x1a0), 0x2483d06975c3965d3f2d205ddeff196b90ca5883878bffc0bd190a357fee947e) // vk.Q_M.y
            mstore(add(_vk, 0x1c0), 0x09af8fed71838d47b0052d8e3fdda11f55c62a6f2cb9aab24edd90b5e9640e9c) // vk.Q_C.x
            mstore(add(_vk, 0x1e0), 0x2bdf7549fa146188dd750d032d9dec911c5799ca99f72405c4ac49f3f9e3a51a) // vk.Q_C.y
            mstore(add(_vk, 0x200), 0x1479a535c87c413301d82c5ae1598b46c03117a57b878416d1143bb48f1df8bf) // vk.Q_ARITHMETIC.x
            mstore(add(_vk, 0x220), 0x03203e3c02cc68282d93507d0ad9d56304d5a4b2908233bcb6f8682f8b264532) // vk.Q_ARITHMETIC.y
            mstore(add(_vk, 0x240), 0x0cccd1de3f4ef2a2bfffbb7a91f8be2c49e9dc9b565ba4312015a88558f40d20) // vk.QSORT.x
            mstore(add(_vk, 0x260), 0x092c5bd4edb996d6c1189a2682f6e93ede4b9aff7f07823605c894f833316718) // vk.QSORT.y
            mstore(add(_vk, 0x280), 0x20089848d81ee4e8d7700679e7b5ed017916e2ee28bf76c0e0f4862274637bb8) // vk.Q_ELLIPTIC.x
            mstore(add(_vk, 0x2a0), 0x0faae100924d24a70708e49a08ba2ba9df261088bf04e7b4c3f811cc0d8995fe) // vk.Q_ELLIPTIC.y
            mstore(add(_vk, 0x2c0), 0x2de71f46452329536fe14dfff808692c405b9ef1ae47c451be8383ded868af5c) // vk.Q_AUX.x
            mstore(add(_vk, 0x2e0), 0x0a520e2f877f19cc69aad2396bf741e6864a9f0b657887e80165b794f7612e71) // vk.Q_AUX.y
            mstore(add(_vk, 0x300), 0x2779b1b7b8433eeee7333a1372feb4587da74e2c93cc54917e201748ed847204) // vk.SIGMA1.x
            mstore(add(_vk, 0x320), 0x2198823f66ad59612f6cb77aff9437388abdbcc4d8f6eac792d8bca7d1b341d9) // vk.SIGMA1.y
            mstore(add(_vk, 0x340), 0x1f6732b9d128931b2e32b2cae73b029720cca3cef23fee25363d520ed0ba3f92) // vk.SIGMA2.x
            mstore(add(_vk, 0x360), 0x15fb336844e68b08361c10b83e7d6ea0f011958774e58e5f7c43e6606e989ecc) // vk.SIGMA2.y
            mstore(add(_vk, 0x380), 0x0984b1b6c723afb4713656abf30b06e2ad04c054dd3acf016a6db1ee7111ca11) // vk.SIGMA3.x
            mstore(add(_vk, 0x3a0), 0x03421d01f19c6b91e477648819f57d888b3b23b67599266293bddf91a2636ff1) // vk.SIGMA3.y
            mstore(add(_vk, 0x3c0), 0x2f77cda90d366b151b17c5667f10526ab0fe144aecb307e00ede6039365bcfa0) // vk.SIGMA4.x
            mstore(add(_vk, 0x3e0), 0x0d1e8f758babcbbf134dfe341c262ee25d0254cba8f5487ad5bddd190f27a9e8) // vk.SIGMA4.y
            mstore(add(_vk, 0x400), 0x2f61a890b9f1dff4ef5c8b0eafe9b71c7a23dc4c8a6791d9c01418310f4a7b2e) // vk.TABLE1.x
            mstore(add(_vk, 0x420), 0x07c8a51d1881fcdfe1cb7dcefc48a44047c7f5386797d5f8553ce2e12e8daba0) // vk.TABLE1.y
            mstore(add(_vk, 0x440), 0x1adf56913dea23b7b14c952933b0b40fc476dc2697a758ec9df73802b0596c2f) // vk.TABLE2.x
            mstore(add(_vk, 0x460), 0x212a1759e19285a35a70a245cca6477f89b6f156e4425cf52cfccb4594f59152) // vk.TABLE2.y
            mstore(add(_vk, 0x480), 0x1527f8c19085ac209ebddbccae4dd0ca58b078e56fd20d651ce3a3194697b191) // vk.TABLE3.x
            mstore(add(_vk, 0x4a0), 0x02247dca9c3cb09318aa6100a2a7c628281c69bc41cfda34aa72c263b69344b4) // vk.TABLE3.y
            mstore(add(_vk, 0x4c0), 0x12eea56d2ada3befa5db215ea5ebbd37b5ce95fcd1cf7adb94d5a1784876b4f7) // vk.TABLE4.x
            mstore(add(_vk, 0x4e0), 0x190df1146fbdd5cc79e8817ebcd6311e35cf5cc38795cee26371a707d685e05a) // vk.TABLE4.y
            mstore(add(_vk, 0x500), 0x019b3a1970f9f77b13538cd8071ea3ee7c556fd98009e2a04be044ead0a94623) // vk.TABLE_TYPE.x
            mstore(add(_vk, 0x520), 0x159cbdae3e194fe45524a171befdcb98b55c8d495fc463c98ac690eee947119f) // vk.TABLE_TYPE.y
            mstore(add(_vk, 0x540), 0x16b2f7fa29f578aae3d4c0b8220101570adfcc9e8aa8a148267208540de189f1) // vk.ID1.x
            mstore(add(_vk, 0x560), 0x2344a211fbbacc281de980197e4f12155d90d55a67f4ad08398bac665f813953) // vk.ID1.y
            mstore(add(_vk, 0x580), 0x1af709df675db1688b95927324e71c5e551436ba7cb32478570a9cfaebf90614) // vk.ID2.x
            mstore(add(_vk, 0x5a0), 0x2b83e76f61aa5cd70218c38e693ae0a99e9a2f4a192af5c77dbd27fa605fdae4) // vk.ID2.y
            mstore(add(_vk, 0x5c0), 0x038c89635a8b6ec9766d5f98d13c16f8c312088f830610de72c00edf8c3b7800) // vk.ID3.x
            mstore(add(_vk, 0x5e0), 0x1863d9217ba6c6764fa02298efe25fabfbe454a27431b970a6afff5d1986fadb) // vk.ID3.y
            mstore(add(_vk, 0x600), 0x259a5dd47d44d6240407c26718201a122fb4b6b38d838f6e24d1c75515016761) // vk.ID4.x
            mstore(add(_vk, 0x620), 0x14db344b735ffe084107e5cea07b00e4c41a82f0073f76e0536cd7118d78866f) // vk.ID4.y
            mstore(add(_vk, 0x640), 0x01) // vk.contains_recursive_proof
            mstore(add(_vk, 0x660), 0) // vk.recursive_proof_public_input_indices
            mstore(add(_vk, 0x680), 0x260e01b251f6f1c7e7ff4e580791dee8ea51d87a358e038b4efe30fac09383c1) // vk.g2_x.X.c1
            mstore(add(_vk, 0x6a0), 0x0118c4d5b837bcc2bc89b5b398b5974e9f5944073b32078b7e231fec938883b0) // vk.g2_x.X.c0
            mstore(add(_vk, 0x6c0), 0x04fc6369f7110fe3d25156c1bb9a72859cf2a04641f99ba4ee413c80da6a5fe4) // vk.g2_x.Y.c1
            mstore(add(_vk, 0x6e0), 0x22febda3c0c0632a56475b4214e5615e11e6dd3f96e6cea2854a87d4dacc5e55) // vk.g2_x.Y.c0
            mstore(_omegaInverseLoc, 0x06e402c0a314fb67a15cf806664ae1b722dbc0efe66e6c81d98f9924ca535321) // vk.work_root_inverse
        }
    }
}
