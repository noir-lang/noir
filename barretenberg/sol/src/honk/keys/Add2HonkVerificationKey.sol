// SPDX-License-Identifier: Apache-2.0
// Copyright 2022 Aztec
pragma solidity >=0.8.21;

import {Honk} from "../HonkTypes.sol";

uint256 constant N = 32;
uint256 constant LOG_N = 5;
uint256 constant NUMBER_OF_PUBLIC_INPUTS = 3;

library Add2HonkVerificationKey {
    function loadVerificationKey() internal pure returns (Honk.VerificationKey memory) {
        Honk.VerificationKey memory vk = Honk.VerificationKey({
            circuitSize: uint256(32),
            logCircuitSize: uint256(5),
            publicInputsSize: uint256(3),
            ql: Honk.G1Point({
                x: uint256(0x043d063b130adfb37342af45d0155a28edd1a7e46c840d9c943fdf45521c64ce),
                y: uint256(0x261522c4089330646aff96736194949330952ae74c573d1686d9cb4a00733854)
            }),
            qr: Honk.G1Point({
                x: uint256(0x291338e99e7857222c76c5e4ba8b954f5fde09fd2f05634d622ba379657cd501),
                y: uint256(0x137030ce3236d7c12307adf650a73b87fc95a774ec43ac0a3a341ef26b7f56c9)
            }),
            qo: Honk.G1Point({
                x: uint256(0x0f90f4bb16b330b82ef51e7ce3f70a9310ea2d3c5ef855f07b6f58081b5ef41f),
                y: uint256(0x0e09412eea75978da57db1d3fa6b7d14c0e282c378be9a6d0efc5770863ed70b)
            }),
            q4: Honk.G1Point({
                x: uint256(0x1eec247154ced5c29b0836528d7c19eda11399dc21e23df4bee4b5cd0bec659f),
                y: uint256(0x107cc382fdee2f6530d39b072a2bc50bdb0c0ac4b054a905b03b9d53bebef404)
            }),
            qm: Honk.G1Point({
                x: uint256(0x0c17b7ba3864cabe287a2b121b5cb3f8ee4ede87a7f656b8d9b470be025007c8),
                y: uint256(0x09590397bf354089980bd40f5d84f4c12faa8b4646425fa660ab7c4c76fb4859)
            }),
            qc: Honk.G1Point({
                x: uint256(0x2ac1a00b4c9bb4e7deef8d7a6bf9e26e61f2b935409e41c5770c074303b6d142),
                y: uint256(0x192d962de288fb26f3d68052b2f475e884ca47e595de1184171cd1500249fa66)
            }),
            qArith: Honk.G1Point({
                x: uint256(0x1797e3e7ee9e4f42b42bd375f13f2ccb395b827e9079e999b6c128d9b083c395),
                y: uint256(0x101a60efaab1c8564add45d41b9147efacf45941c3efe93c3568bde1e08e1919)
            }),
            qDeltaRange: Honk.G1Point({
                x: uint256(0x0e84090add56f2500ab518c655cae63896ea793e6b3f6a14218d476534109610),
                y: uint256(0x2b78a584bd6ae88cf4ec7c65c90e0b65df446fdddba972f3c4414ad3c901f4f9)
            }),
            qElliptic: Honk.G1Point({
                x: uint256(0x1bd6129f9646aa21af0d77e7b1cc9794e611b5d59a27773f744710b476fbd30f),
                y: uint256(0x2f8d492d76a22b6834f0b88e2d4096139a9d1593d56e65e710b2f344756b721e)
            }),
            qAux: Honk.G1Point({
                x: uint256(0x056ab50282da428d93b17cbd1c81267dcebcfbabdedb47b2d715b5baa6520bff),
                y: uint256(0x10b4e7bd9d6d91a57b0695be166ffd27cbeee602bcb5a9ed32c8d9440912cb72)
            }),
            qLookup: Honk.G1Point({
                x: uint256(0x19e2d786ebad24caf1bef735441e58525a2f9b5807b2102f295c58cde00f5c97),
                y: uint256(0x085713ce7bac807a084a66904ebc6e695840e8cf405a6fd0c325f8bfcf7c2dd8)
            }),
            s1: Honk.G1Point({
                x: uint256(0x039e4b13c22e227df79cafc6a8a9e8cc6217791d738ccad75c88d4a150cf9324),
                y: uint256(0x16b832f3a75a8dffdb969bb79918867eaff198957c687d20ebc726dcbd61f9e1)
            }),
            s2: Honk.G1Point({
                x: uint256(0x165d5d53619ae0694e61c26302abfe39bc20646fd04210519520980f3ffb91d2),
                y: uint256(0x10f0c4a0b216e66fe2f88ae617b54347fa41fce598ec801dd289394890f3b66b)
            }),
            s3: Honk.G1Point({
                x: uint256(0x0afca124a27006abfd194b1ad23b404f2112d11fe95ec69cc705f02258adc913),
                y: uint256(0x2c1acafafadacc271d5297ab45bc79a5a3c87916578d8309493c2e2507bbe508)
            }),
            s4: Honk.G1Point({
                x: uint256(0x0f7cf2c8c0cd0fe37630e8f79ba6dffd59c0fd1f2f6ae5789efab86d4c1c0007),
                y: uint256(0x25bde264a751e99cf36bb64034710e55b3b1e02234c75a35683c91274d445ed8)
            }),
            t1: Honk.G1Point({
                x: uint256(0x2e0cddbc5712d79b59cb3b41ebbcdd494997477ab161763e46601d95844837ef),
                y: uint256(0x303126892f664d8d505964d14315ec426db4c64531d350750df62dbbc41a1bd9)
            }),
            t2: Honk.G1Point({
                x: uint256(0x00874a5ad262eecc6b565e0b08507476a6b2c6040c0c62bd59acfe3e3e125672),
                y: uint256(0x127b2a745a1b74968c3edc18982b9bef082fb517183c9c6841c2b8ef2ca1df04)
            }),
            t3: Honk.G1Point({
                x: uint256(0x15a18748490ff4c2b1871081954e86c9efd4f8c3d56e1eb23d789a8f710d5be6),
                y: uint256(0x2097c84955059442a95df075833071a0011ef987dc016ab110eacd554a1d8bbf)
            }),
            t4: Honk.G1Point({
                x: uint256(0x2aecd48089890ea0798eb952c66824d38e9426ad3085b68b00a93c17897c2877),
                y: uint256(0x1216bdb2f0d961bb8a7a23331d215078d8a9ce405ce559f441f2e71477ff3ddb)
            }),
            id1: Honk.G1Point({
                x: uint256(0x292298ecab24d2b6f6999cac29848def2665a62342170311f44c08708db0fe1f),
                y: uint256(0x277022c35d3145de166b139aa94609551122915366ba42ff7c5157b748fb7f9d)
            }),
            id2: Honk.G1Point({
                x: uint256(0x2ddc6a05ccd584bdfc65d642b39a3be3075e7a370602112dbf9fc644789acace),
                y: uint256(0x1a4167481d5f295af9921741bd0e32dda7a78cb391132b31ab4a77559c297c2e)
            }),
            id3: Honk.G1Point({
                x: uint256(0x19629b85ab2acf9713223ff4f758882af6247963bbf2f6ec4f9cbcde13675b87),
                y: uint256(0x165063fe922948bf1d065a882242724c1bde5fdfd93be29586b45e1ce2cc750c)
            }),
            id4: Honk.G1Point({
                x: uint256(0x2493c99a3d068b03f8f2b8d28b57cea3ee22dd60456277b86c32a18982dcb185),
                y: uint256(0x1ded39c4c8366469843cd63f09ecacf6c3731486320082c20ec71bbdc92196c1)
            }),
            lagrangeFirst: Honk.G1Point({
                x: uint256(0x0000000000000000000000000000000000000000000000000000000000000001),
                y: uint256(0x0000000000000000000000000000000000000000000000000000000000000002)
            }),
            lagrangeLast: Honk.G1Point({
                x: uint256(0x140b0936c323fd2471155617b6af56ee40d90bea71fba7a412dd61fcf34e8ceb),
                y: uint256(0x2b6c10790a5f6631c87d652e059df42b90071823185c5ff8e440fd3d73b6fefc)
            })
        });
        return vk;
    }
}
