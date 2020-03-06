#include <gtest/gtest.h>

#include <barretenberg/curves/bn254/fq12.hpp>

using namespace barretenberg;

TEST(fq12, eq)
{
    fq12 a = { { { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
                 { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
                 { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } } },
               { { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
                 { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
                 { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } } } };
    fq12 b = { { { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
                 { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
                 { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } } },
               { { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
                 { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
                 { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } } } };
    fq12 c = { { { { 0x01, 0x02, 0x03, 0x05 }, { 0x06, 0x07, 0x08, 0x09 } },
                 { { 0x01, 0x02, 0x03, 0x05 }, { 0x06, 0x07, 0x08, 0x09 } },
                 { { 0x01, 0x02, 0x03, 0x05 }, { 0x06, 0x07, 0x08, 0x09 } } },
               { { { 0x01, 0x02, 0x03, 0x05 }, { 0x06, 0x07, 0x08, 0x09 } },
                 { { 0x01, 0x02, 0x03, 0x05 }, { 0x06, 0x07, 0x08, 0x09 } },
                 { { 0x01, 0x02, 0x03, 0x05 }, { 0x06, 0x07, 0x08, 0x09 } } } };
    fq12 d = { { { { 0x01, 0x02, 0x04, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
                 { { 0x01, 0x02, 0x04, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
                 { { 0x01, 0x02, 0x04, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } } },
               { { { 0x01, 0x02, 0x04, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
                 { { 0x01, 0x02, 0x04, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
                 { { 0x01, 0x02, 0x04, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } } } };
    fq12 e = { { { { 0x01, 0x03, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
                 { { 0x01, 0x03, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
                 { { 0x01, 0x03, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } } },
               { { { 0x01, 0x03, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
                 { { 0x01, 0x03, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
                 { { 0x01, 0x03, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } } } };
    fq12 f = { { { { 0x02, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
                 { { 0x02, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
                 { { 0x02, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } } },
               { { { 0x02, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
                 { { 0x02, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
                 { { 0x02, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } } } };
    fq12 g = { { { { 0x01, 0x02, 0x03, 0x04 }, { 0x07, 0x07, 0x08, 0x09 } },
                 { { 0x01, 0x02, 0x03, 0x04 }, { 0x07, 0x07, 0x08, 0x09 } },
                 { { 0x01, 0x02, 0x03, 0x04 }, { 0x07, 0x07, 0x08, 0x09 } } },
               { { { 0x01, 0x02, 0x03, 0x04 }, { 0x07, 0x07, 0x08, 0x09 } },
                 { { 0x01, 0x02, 0x03, 0x04 }, { 0x07, 0x07, 0x08, 0x09 } },
                 { { 0x01, 0x02, 0x03, 0x04 }, { 0x07, 0x07, 0x08, 0x09 } } } };
    fq12 h = { { { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x08, 0x08, 0x09 } },
                 { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x08, 0x08, 0x09 } },
                 { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x08, 0x08, 0x09 } } },
               { { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x08, 0x08, 0x09 } },
                 { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x08, 0x08, 0x09 } },
                 { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x08, 0x08, 0x09 } } } };
    fq12 i = { { { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x09, 0x09 } },
                 { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x09, 0x09 } },
                 { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x09, 0x09 } } },
               { { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x09, 0x09 } },
                 { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x09, 0x09 } },
                 { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x09, 0x09 } } } };
    fq12 j = { { { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x0a } },
                 { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x0a } },
                 { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x0a } } },
               { { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x0a } },
                 { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x0a } },
                 { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x0a } } } };

    EXPECT_EQ((a == b), true);
    EXPECT_EQ((a == c), false);
    EXPECT_EQ((a == d), false);
    EXPECT_EQ((a == e), false);
    EXPECT_EQ((a == f), false);
    EXPECT_EQ((a == g), false);
    EXPECT_EQ((a == h), false);
    EXPECT_EQ((a == i), false);
    EXPECT_EQ((a == j), false);
}

TEST(fq12, is_zero)
{
    fq12 a = fq12::zero();
    fq12 b = fq12::zero();
    fq12 c = fq12::zero();
    b.c0.c0.c0.data[0] = 1;
    c.c1.c0.c0.data[0] = 1;
    EXPECT_EQ(a.is_zero(), true);
    EXPECT_EQ(b.is_zero(), false);
    EXPECT_EQ(c.is_zero(), false);
}

TEST(fq12, random_element)
{
    fq12 a = fq12::random_element();
    fq12 b = fq12::random_element();

    EXPECT_EQ(a == b, false);
    EXPECT_EQ(a.is_zero(), false);
    EXPECT_EQ(a.is_zero(), false);
}

TEST(fq12, add_check_against_constants)
{
    fq12 a = { { { { 0xe5090b4f4ae647a8, 0xf5d4801f152fdf6c, 0xcdb69d33dba7f562, 0x228f26abab7d6687 },
                   { 0xc27a82b14db8404f, 0xcbf9354b3655de9b, 0xa57fd51d8df378ad, 0x2e3fc75bde967502 } },
                 { { 0x68313cbef88a5759, 0x5b133f7951386452, 0x39e7dc61e0a99225, 0xd431d506584ef0c },
                   { 0xe1cb4e7cdde02d54, 0x77850ffc86025996, 0x32f2ac7001b781bc, 0x161e5d998a9ca841 } },
                 { { 0x6a19408ab0b98992, 0xdf60d135d3ca1b8e, 0xc2c4fb314a6fb787, 0x6dad3e4fcdc4109 },
                   { 0xf19bf45ccc4fa4c7, 0x772e75432f1a1b7d, 0xf5fdbc43092cb02b, 0x1e8b68995e7650a4 } } },
               { { { 0xbb590c3f964679e8, 0x7cc101ff81317c47, 0xf0795826878a3d87, 0x2fa65099a1bb9d0a },
                   { 0x366de96875edef88, 0xb912fe4346bce97f, 0x9021ce3f941f43ee, 0x1df070ec2d40307e } },
                 { { 0x2f374251a079ed16, 0x781ed5eba2c32177, 0x46f730607db6faaa, 0x17059595d4861d63 },
                   { 0xffac861b22b6af3c, 0x2d42efdd21ffbfa, 0xcc63ae5632de7dc4, 0x303860faf42e67d4 } },
                 { { 0x83ee1b92ac44538, 0x2ad45ea546f39c5e, 0x983243f0954f3d85, 0x2cda1dee8630e07d },
                   { 0xdf1ea9591f4f9acc, 0x967ee067573afba1, 0x43b75fdd61c3a0f9, 0x7a3fcd3793426e2 } } } };
    fq12 b = { { { { 0x245aaeeeca2e8585, 0xdf288bfa5a851ba8, 0x735d09b2b32bea1e, 0x28086604acb87f71 },
                   { 0x997ea2f96d830756, 0xc59e9aed4c05f014, 0xf47897613190565a, 0x25e98544dc2c8831 } },
                 { { 0x3a03ed7df625906d, 0x85650c015c4c8d83, 0x40acaeaff65a8f65, 0x1a7c84a9f7b973ca },
                   { 0xc30774c5f550ab0a, 0x6cab91e0a92ca253, 0xef9a802117ecb2e6, 0x11fe101c5c74dd84 } },
                 { { 0x4fc81f6d823665, 0x474c3b6df0c616ae, 0xcfc039d881f4dbc5, 0x13f25378fc4b7321 },
                   { 0x48be6e134dfb42e1, 0x13242e5605a54db6, 0x3a3ac4c510c6d51d, 0x1d828e5782e808c7 } } },
               { { { 0x4d5aaa2e6bf2f6ce, 0x77b940cf8467d6bd, 0x3db11f4ecb5d955f, 0x26c0713713247292 },
                   { 0x189f56d466889ee9, 0x71c8f6408d8cc0, 0x3ce56be0a825f633, 0x1dda277a21fc4a5 } },
                 { { 0x3385740320b58676, 0x22acfbf39a2b78ea, 0xcbc2d6fbc8b3627e, 0x153c7d04d6a74399 },
                   { 0x6e34eb6dd8178662, 0x80cb55dc8f40d064, 0xeb7ec75be44cb60c, 0x2b2ab9ce7d0e325b } },
                 { { 0xa12ef4f30cba6dcc, 0x2a75211f0179f708, 0x805c5f02cea31575, 0x33b07ab9f23b37f },
                   { 0xdfc83826a0a99b10, 0x44e813c27143ff65, 0x44006acae2a0f4b2, 0xda7d72fd6d6b8be } } } };
    fq12 expected = { { { { 0xcd432e273c97cfe6, 0x3d7ba18807433087, 0x88c361300d528724, 0x1a333e3d770445cf },
                          { 0x1fd89993e2be4a5e, 0xfa1665a719ea0423, 0xe1a826c83e0276aa, 0x23c4fe2dd9915d0a } },
                        { { 0xa2352a3ceeafe7c6, 0xe0784b7aad84f1d5, 0x7a948b11d704218a, 0x27bfa1fa5d3e62d6 },
                          { 0xa4d2c342d330d85e, 0xe430a1dd2f2efbea, 0x228d2c9119a434a2, 0x281c6db5e71185c6 } },
                        { { 0x6a6908aa1e3bbff7, 0x26ad0ca3c490323c, 0x92853509cc64934d, 0x1acd275df927b42b },
                          { 0xfe39d65941cdea61, 0xf2d13907cc4d9ea6, 0x77e83b5198722cea, 0xba9a87e002cb942 } } },
                      { { { 0xcc932a5729bc736f, 0x5cf8d83d9d278877, 0x75da31bed1667a89, 0x2602735dd3ae6f73 },
                          { 0x4f0d403cdc768e71, 0xb984c739874a763f, 0xcd073a203c453a21, 0x1fce1363cf5ff523 } },
                        { { 0x62bcb654c12f738c, 0x9acbd1df3cee9a61, 0x12ba075c466a5d28, 0x2c42129aab2d60fd },
                          { 0x31c0e57222513857, 0xec1e1a48f8ef01d2, 0xff922ffb95a9db72, 0x2afecc56900afa06 } },
                        { { 0xa96dd6ac377eb304, 0x55497fc4486d9366, 0x188ea2f363f252fa, 0x3015259a255493fd },
                          { 0xbee6e17fbff935dc, 0xdb66f429c87efb07, 0x87b7caa8446495ab, 0x154bd403500adfa0 } } } };
    fq12 result = a + b;
    EXPECT_EQ(result, expected);
}

TEST(fq12, sub_check_against_constants)
{
    fq12 a = { { { { 0xf828470cae144505, 0xe05f8f664caae877, 0x27b46814f04c96a3, 0x49d8f97c040a1a2 },
                   { 0x3651629333fd6d1, 0xf7c08d56035cb892, 0x7fd937c7d75b567f, 0x11aac5d9567d8c7e } },
                 { { 0xd47a8bb08e4a676f, 0x4e8ec845cea67faa, 0xb643308828d68eca, 0xd707220f70bb972 },
                   { 0xbde9a346a4e047e2, 0xe591795a7234509, 0x25f51ea67778b6a7, 0x1ae4992ae87a5275 } },
                 { { 0x3a18d8e1bb596051, 0x98a5e65ed32f9a9d, 0xaa7e1d1c2f37f724, 0x1c49e3f27295d2ad },
                   { 0x474045a3fc2d57bf, 0x97589ff46e5fb00d, 0x61d9e3d73384db69, 0x131a19f4298c9de2 } } },
               { { { 0x706133429743b32, 0x1e6447ce7c8339f8, 0x298b9e4c899c0bd7, 0xafa684968b50282 },
                   { 0xcc455561541a8cee, 0xfdf2f5fc63431236, 0xe07e5b31b4d6b6e5, 0x1be630fe3cb76f18 } },
                 { { 0xd43cd3daf76a5feb, 0x435676c0cb9e6de0, 0x4a33cda2a7256295, 0xf72cd4603cc9157 },
                   { 0xe459117a2159f829, 0x79e9077fdc67694d, 0x2e9455f7048f57c9, 0x2db6d0e0acacbefd } },
                 { { 0x3de184f574e6a906, 0xa76c351979057c33, 0xaddd00ad7f01ef80, 0x27ee592a59c8bc64 },
                   { 0x51a52b09f0ce135d, 0xac3d8564b56af445, 0x84991dd9a2667ea5, 0x21d842ca48429246 } } } };
    fq12 b = { { { { 0x93d4916d22adcba0, 0x683e80bb29e3ec60, 0x388f10e59e129cfb, 0x19e09f077f0675ce },
                   { 0xf1a9f3b587b1e349, 0xd6a6887757b799b0, 0x95d9dee7828e0465, 0x6aff335face1944 } },
                 { { 0x6b756aa7799e83d1, 0x6369ec6364c2929d, 0xaa05da6983b0c4a3, 0x13adec60e10fddb5 },
                   { 0x138177f49f63310, 0x2d4da5c584a5becb, 0x807fc7d23a607feb, 0x1e6357a0ab0d3670 } },
                 { { 0xab12964a122cf1b6, 0x99ef88e109b9d21a, 0x32215e4c634251b1, 0x24717e2ca7747434 },
                   { 0x3cd50c8d4c4095d2, 0x18b12fa8ddc10828, 0xe5bb788ac1476aa6, 0x2d91a0bc1e3f42b3 } } },
               { { { 0x237a667d2dd96f73, 0xfefb62c91bee98ef, 0xe49892737da80da0, 0x28df2b3a12aeeac3 },
                   { 0x5fb1f89589c428df, 0x6de7c766737bec95, 0xa70a6430b61b7bbc, 0x2cf8498d167b3c80 } },
                 { { 0xa26a8dab924ff949, 0xdbbe720bad96b613, 0x8b1af40cdfc70a62, 0x1d75af668d1a5e75 },
                   { 0x9b716b611633cb21, 0xf5495a0ba4b827da, 0xfa13f0e39a7c4d80, 0x1efa635ab063db6f } },
                 { { 0x3d6d5d0a4a1add14, 0x8168927ff59bd9aa, 0xc7ca1b1129a41303, 0x2541c1d8afe4a234 },
                   { 0xbea0fc047742181b, 0x302f7d5a9e270655, 0x83259d4a1a636c93, 0x231cc5a6fecd70ff } } } };
    fq12 expected = { { { { 0xa07441b663e376ac, 0xfa2793c8b38c6a4, 0xa7759ce5d3bb5206, 0x1b213f03226bcbfd },
                          { 0x11bb2273ab8df388, 0x211a04deaba51ee1, 0xe9ff58e054cd521a, 0xafad2a35baf7339 } },
                        { { 0xa525ad1fed28e0e5, 0x82a64673d255b79a, 0xc48d9bd526a72284, 0x2a26d432f72d7be6 },
                          { 0xf8d217de33671219, 0x788cdc618aef50cb, 0x5dc59c8abe998f19, 0x2ce58ffd1e9ebc2e } },
                        { { 0xcb26ceae81a96be2, 0x9637c80f31e7930f, 0x30ad04864d76fdd0, 0x283cb438ac52fea3 },
                          { 0x468bc52d8869bf34, 0x1628dadcf9107272, 0x346eb102f3bec921, 0x15ecc7aaec7efb58 } } },
                      { { { 0x1fac38cdd417c906, 0xb6ea4f96c9066b96, 0xfd43518f8d755693, 0x127f8b823737b7e7 },
                          { 0xa8b3e8e2a2d36156, 0x278c99275838f02e, 0xf1c43cb7803c9387, 0x1f5235e4076dd2c1 } },
                        { { 0x6df2d2463d9763e9, 0xff196f468679825a, 0x77691f4c48dfb08f, 0x22616c5257e3d30b },
                          { 0x48e7a6190b262d08, 0x849fad7437af4173, 0x348065136a130a48, 0xebc6d85fc48e38d } },
                        { { 0x7427eb2acbcbf2, 0x2603a2998369a289, 0xe612e59c555ddc7d, 0x2ac9751a9e41a2f },
                          { 0xcf24bb1c5208f889, 0x138f729b7fb5b87c, 0xb9c3c64609846a70, 0x2f1fcb962aa6c170 } } } };
    fq12 result = a - b;
    EXPECT_EQ(result, expected);
}

TEST(fq12, mul_check_against_constants)
{
    fq12 a = { { { { 0xd43e9f8be859502b, 0x26a42a1a95cee1ef, 0x3d63c085c1892b32, 0x2e5beaf431211a76 },
                   { 0x5f32ad7cee215ff5, 0xce967fda9424120e, 0x10ea4e52628bac33, 0x51b85ee9671b7f3 } },
                 { { 0x95f8e84e0ff94a83, 0x6c6fb2cf3c73b30a, 0x28e8e13841f714a8, 0x2a3412f681e31b4d },
                   { 0xcd03b28cdc6fcc09, 0xf7ee307ef2077412, 0xbc9d525a4ffd0836, 0x2d4f5f6e688bf0dd } },
                 { { 0xb1908010bcb66716, 0xbd23aa28e73d7e20, 0xfc3464edca7f2034, 0x18df61620d6a3cc1 },
                   { 0x56720acb51310603, 0x9a77e402b6e5a115, 0xe35197b7788364cd, 0x2e6e4007d35342d7 } } },
               { { { 0x33ec11b94eac5710, 0x73c9dcd8bdea426d, 0xeeb51979ffe73f42, 0x9cc4ada6e7c8b3c },
                   { 0xcfa8f5fe99105971, 0x32f56cb5227e01fa, 0xfcbf8257d846eaa6, 0x6d61581ae78a315 } },
                 { { 0x242e46e642ade123, 0xc5de69ea316cbe0f, 0x63085993f3770f26, 0x9d2ffea4b8e13e1 },
                   { 0x13eddc3b1cf3d2f1, 0xb79750aed7439222, 0x6615c429a49c7b6e, 0x243e8df9e6214e9c } },
                 { { 0x82c1aefce33671b, 0x46d36904f9839aaa, 0x37f089bd51cd0269, 0x207b0fbf328c0c1b },
                   { 0x408aa3eafba634f9, 0x7dfc25c916d2a990, 0xb9b9afe508057cb2, 0x10045453df51156b } } } };
    fq12 b = { { { { 0xc544906e527a4dbb, 0x3b08eba65d831ce2, 0xf44924359d69ef21, 0xeb24efb0d86c18d },
                   { 0x762256ecff65aa30, 0xcc5d8240a745dd40, 0xc9ae36345a8d3a7, 0x23aae88e4c37875c } },
                 { { 0xdcbd0dbc20d6245f, 0x54f64b8ceeebe3e9, 0xc2f8d8dcd678c88e, 0x908df13ed6053d1 },
                   { 0x4923d2490485825a, 0xf1a9109e0b644eb5, 0x429d1b8bacc163f6, 0x15013536538d4f45 } },
                 { { 0x659fa4d073b4406, 0x222fce98d820bc19, 0x68e5ba5f6a6b0185, 0x2a76a4a0a34f20b2 },
                   { 0xfa94b15348a564dd, 0x28f6070c9cc9d3e6, 0x2d51cbd531b80d38, 0x90238942d0598dd } } },
               { { { 0x774b857008f9eb46, 0xaa79c6612b259dd3, 0xaaf7799629c36c25, 0x199674aa3fc4dc4 },
                   { 0x521ed8b0f4029e01, 0xb5cef10d7e9082a7, 0x22a2dbf23ddbfdc7, 0x10ed71ea96a11df8 } },
                 { { 0xfcc59d9a0a13d0da, 0x406533bb1dec3d99, 0x6b8aa2d76bcc954b, 0x217785904ce22f1a },
                   { 0xa65e26fd5fc187af, 0xf09cff1136f9ef97, 0xecbb91eec77033ef, 0x1e24cd13205cc6a1 } },
                 { { 0xcbdde8b25e2f3316, 0x92ace6648cdee119, 0x2578759b6d80407b, 0x73b0d667c920360 },
                   { 0x37012fc28dfb6150, 0xbbbe148fa2b455ba, 0x3e40c8247f10d3b5, 0x2659914dcf14d694 } } } };
    fq12 expected = { { { { 0x90705f5d2661c46a, 0x8359404686aca366, 0xf1746b64a8e7726f, 0x8f5986313948a97 },
                          { 0x27bede3683dfbbee, 0x7b848bc1f4e2538f, 0xf00da00ebe5a4978, 0x2d936f521ae05a79 } },
                        { { 0x43be456f10e7479, 0xa213a8ec67a82f12, 0x1ab6e831782ea23c, 0x22b9ad870683188 },
                          { 0xc62073ee2256503b, 0x5b2c57b0384c5915, 0xc62b200365d81355, 0x77e010f605b0fa0 } },
                        { { 0x1ef104ada95be821, 0x25519a0c57a40a9d, 0x729f93a79a60faea, 0x15f0ba49c2ffc141 },
                          { 0x9f0e325fa9e3aae5, 0xc2b6acd6f85edf02, 0xc490de65594e8f0, 0x2a5589155f78b7d6 } } },
                      { { { 0x3d974df017795d21, 0x11abc3064ac33706, 0xbfdcf2eed81ed0d0, 0x2c880426e5fc3264 },
                          { 0x44db12fb914f9b11, 0xe5e47f8a30738ef4, 0x7088a7a22dfb33c3, 0x1a992c655dc96df9 } },
                        { { 0x911391f2830c808f, 0x304ef4bf1ba305f9, 0xe449bc05ed945fa1, 0x27443aad475e4349 },
                          { 0xeaf256aa7a6b49b5, 0xeaa1b56258e3194e, 0xde3b531fd4fe961b, 0x26a0b5c35ce4be53 } },
                        { { 0x1f7661fa7dd7d68c, 0x71c1360fdb272200, 0x3fdb8fcc1dbfd160, 0x1ba330295e24399b },
                          { 0x5c93a291c6579918, 0x6536baab9e09bc80, 0x93ad9959edff4c64, 0x138af9a14abfeb1e } } } };
    fq12 result = a * b;
    EXPECT_EQ(result, expected);
}

TEST(fq12, sparse_mul_check_against_constants)
{
    fq12 a = { { { { 0x8860ba0c4eea41a8, 0x71b65207984d47d2, 0x67e55696f8982ba9, 0x18236f03bcec9b00 },
                   { 0xa69e0f0ce60f64fd, 0x1cf52f3b2335b9b3, 0x45e8ec475fcb1d71, 0x1627ac08d10cebd9 } },
                 { { 0xc7343ce2fb7b4829, 0xff667dd3e618123b, 0xd03970bcf60881b4, 0x188e0b7acdd0b801 },
                   { 0x49f0e3920f6b1b15, 0x977066c1663507f0, 0x8f936a1db7d7e941, 0x25bf1ece32f248aa } },
                 { { 0x8438caab6d1ecf86, 0x2ed52111d1a6b5cf, 0x86df32501be37e2, 0xf1b28b016bf5f80 },
                   { 0xd44b6c69e9ee39e1, 0x342cd151e96b83cf, 0xaa1636c10ca05d1d, 0x26618a4ff21f4c0c } } },
               { { { 0x25eb0d146d490133, 0x7aac76625adcb396, 0xbb5453d3b720fe1c, 0x13d15501439e2bb1 },
                   { 0x229dcb8f53832e4e, 0xfe66f225581821e6, 0x5d62e2d5750a11bd, 0xc0c88e5db4f2686 } },
                 { { 0x2244c0b1d2914cc1, 0xde2a8091ffc6ebfa, 0x9a93d5013c18da96, 0x1bbf794425605078 },
                   { 0x5752f0197b67dfa3, 0xb4ff7a53c23b98fd, 0x95dec4882eb275cd, 0x6815e3c55e10152 } },
                 { { 0x64f434f52a58b19b, 0xcdab64e3ae898031, 0x5d10a474f28b9462, 0x85452691edf6f18 },
                   { 0x2bb46c10f494b711, 0x66a853baee9e6a00, 0x3b3e0fd932afa021, 0x1ae752d1bbdef131 } } } };
    fq12::ell_coeffs ell;
    ell.o = { { 0xe49c67a74aaf8c22, 0xc5cc428c85da5d5a, 0xc946262e0c99d3d9, 0x2307b236a862e3e9 },
              { 0x1659aef76f0397ef, 0x32d0c2d00f81d8a5, 0x7e87867d5f0c5ccd, 0x247307a3fd6fece7 } };
    ell.vv = { { 0x6e6f2db65bdf07bd, 0xc26fa997848fb1e4, 0x13ec10cb6a0cd0ae, 0xf86d8967480301c },
               { 0xf21de59187942446, 0x276793288a98f5bc, 0xb694797773d6cd4a, 0xad0465ff5d8d6eb } };
    ell.vw = { { 0xd2656d7ad38190f4, 0x727bf9c43fbc0616, 0x8e46c86249f5d1d8, 0x2e0cdaef67d9f1c7 },
               { 0xaee8cf13b08c0fd8, 0x1c57393117f6dd41, 0xe2ab0caf53f3f1fd, 0x26dff49d041e0a7 } };
    fq12 expected = { { { { 0xa138614e32e52b75, 0x36f23cce4b07643d, 0x8c60d007b3418fd7, 0xabfb822e56acf97 },
                          { 0x89db81e8d93d78f, 0xe5823c60873135ba, 0xbf546b50fcaeb66, 0x6c124b404707e06 } },
                        { { 0xc5258919e6e0cf0d, 0x919701db293e74b2, 0xd045941d4fff3b96, 0x22534dde281b7418 },
                          { 0x4f0df00018798b14, 0x6a2b13b3072b6f44, 0x86a6d0c7fb0f0a99, 0xdc5ba43d724139b } },
                        { { 0x45e1143ac84724d, 0x72fa5399fd3fc5ed, 0xbd40b4eafeb3dc4f, 0x2c991c66c3ccde7d },
                          { 0xcb624c488f504a6d, 0x5f6b141cc43311e1, 0x5fbc3067fe228695, 0x1f9895620b420936 } } },
                      { { { 0xede912793f673746, 0x679c963adaa9dec0, 0xbc1eaae42d5af983, 0xc25be9ac14e2d49 },
                          { 0x69015b5d44cc1e5f, 0xd4f45f5a15acf0bc, 0x262082e7757440f6, 0x1ecf7aacf2385fce } },
                        { { 0xb77b1ece77cc5b9c, 0x45bc9c63533cdd8, 0x4e449373e98dfa33, 0x9cc3f93849bed65 },
                          { 0xee892e54b68159d6, 0xe0421cb20d103d69, 0xfe0591fdca60e2e3, 0x1650989fd73116b9 } },
                        { { 0x475dec6d5f2e2a75, 0xf25390f14ed7106, 0x61a4b571cb15d2fe, 0x1ad83abac0d5bdd7 },
                          { 0x8f730272c4cfee79, 0x60833c047d98a040, 0xbd1da3dc3fe5ad4a, 0x11bcc8faf5176d94 } } } };
    a.self_sparse_mul(ell);
    fq12 result = a;
    EXPECT_EQ(result, expected);
}

TEST(fq12, sqr_check_against_constants)
{
    fq12 a = { { { { 0xef9d68a7df0715fd, 0xfda8aff4030523cf, 0xd09b1482069c0972, 0x252195422f351b07 },
                   { 0x3192057a31dec453, 0xe1c2dd8879191e47, 0xe90a8a00c9b29c5b, 0x1db75f06dff5dd5e } },
                 { { 0xdb01b2dbb451df8f, 0x42d8923147ae4171, 0xd1264f3077ab1733, 0x2fbabfe2fbc0c62f },
                   { 0xb942d7f55f2300c6, 0xd5f9c907303a4377, 0x4b738eb660419004, 0x105fd9e8561dde19 } },
                 { { 0xd75f00fb47a5933b, 0x90b2c7e3db87476f, 0xce6e2140c49dfe37, 0x204b393bab70c36a },
                   { 0xe6ccc13d4cb9f2a1, 0x647703faa348c685, 0xcffcc5fab2041de9, 0x26390187897069d4 } } },
               { { { 0xf411e9d666eb1c62, 0x5e79344ceba3c66a, 0x7952b27642ac6fab, 0xd99ea6525d5b1d5 },
                   { 0x684ee7c0845405ab, 0xe7e0a59a8540a44, 0xa78c74e32cbffa52, 0x2701b3cc0496a1e5 } },
                 { { 0xf4efc2a5342a90f0, 0x34fe1ec2efba9bed, 0x6457f324257a5bf6, 0x154f629fd85e3e3f },
                   { 0x1fbea12685ae2c87, 0xe72f6e501b2c85fe, 0x2c3cb81d695bb3e1, 0x2328129a857763fc } },
                 { { 0x5260c233e0adaf77, 0x20a311f1c26ee1d9, 0x6084dae2715116bb, 0x19dc89569a2a6f66 },
                   { 0x246466b86cc89d2c, 0x126f819c3031f783, 0x4eaf2d505a6158d4, 0x1c3a124bbbcb851 } } } };
    fq12 expected = { { { { 0x32705d7cf5a364e3, 0x446995889b6c9278, 0x68016f8f5e05c46c, 0x1538ea0aea917bf9 },
                          { 0x700b02b3212abeba, 0x97881851fbeb8379, 0x2b3772e40c72fa4c, 0x227d63d2149286d0 } },
                        { { 0x4d52012a7af91a46, 0x11d348c55c0f80a0, 0xc7404131a714a543, 0x1b3c367127d42ed5 },
                          { 0x470be7817ba24d95, 0xa24a051e930fd709, 0xa2ecfe1119e0114f, 0x121a2c6bf2023fa3 } },
                        { { 0x34d2687ca37c8f51, 0x45d98e94e67ce9eb, 0xf8cb2850c835c8fb, 0x5f862e96fbd08c6 },
                          { 0xb433ae036220b411, 0xabda439ed34d9e10, 0x172a87ec00dd4588, 0x22d963b58a394b03 } } },
                      { { { 0x47e81dbb3d96dd4a, 0x8f2374b381aec29b, 0x563d859a3117771a, 0x15304bf6eaf07eb5 },
                          { 0xdd7fdbda9b473d87, 0xb29690530d01ba9, 0x31e385e8cb4fe384, 0x156826e46c02f167 } },
                        { { 0xeece14a760655b9c, 0xaeba2fd6595006f3, 0x816376d423c9948a, 0x15949533f02c2dee },
                          { 0x19ea0ed62e5093c2, 0xcf288a69b5a24352, 0xa9bdc89dd4491b7d, 0x447edc7b33f3d1c } },
                        { { 0xceb417494bece8e, 0x7f3d84971a20d351, 0x31679ed74c101d91, 0x1bb2c06842073c0c },
                          { 0x6db2993066e5fd73, 0x2c08c9fd6c3b5483, 0x3b32d43ab22d6cea, 0x3df72d32906f5f0 } } } };
    fq12 result = a.sqr();
    EXPECT_EQ(result, expected);
}

TEST(fq12, inverse)
{
    fq12 input = fq12::random_element();
    fq12 result = input.invert() * input;
    EXPECT_EQ(result, fq12::one());
}

TEST(fq12, unitary_inverse)
{
    fq12 input = fq12::random_element();
    fq12 result = input.unitary_inverse();
    EXPECT_EQ(input.c0, result.c0);
    result.c1 += input.c1;
    EXPECT_EQ(result.c1, fq6::zero());
}

TEST(fq12, frobenius_map_three)
{
    fq12 a = { { { { 0x9a56f1e63b1f0db8, 0xd629a6c847f6cedd, 0x4a179c053a91458b, 0xa84c02b0b6d7470 },
                   { 0xffa3e17eab3609a1, 0x6a97b9cf5c3fe152, 0x8996248da177be9f, 0x113bd2d7f24591d } },
                 { { 0x572c4fd8a85cc3b, 0x48197102a98815e8, 0x3a1d00190e8ee460, 0x8c0a0ce9c093781 },
                   { 0x4e0e0e931a6c5239, 0xc2c764493f1ddc6e, 0x16612bee3c36cb07, 0x1c4bcafbc27e189d } },
                 { { 0x9c25202f11b4a225, 0x6183855884e1d9e4, 0x9bec1271f82069fd, 0x25c073771f7bdfd3 },
                   { 0x8369ec32ca273e66, 0x72abfd9ddb3c9580, 0x45c8c3900fa0972, 0xed4e39f24d881cc } } },
               { { { 0xbe14c75a7ad8b8d, 0xa7f800a3617eb0cc, 0x8ac553d859ebfa82, 0x1bd48369b2897384 },
                   { 0xa87047cda886d4e4, 0xa10f79abb449eca1, 0xa91048654572ca4a, 0x11800c4140b84683 } },
                 { { 0xbdab137f3526b04c, 0xcdf528cca5ac1194, 0xcd0ff308caa11d1a, 0x13c37af89e7ad03c },
                   { 0x9014dab1a6705e3e, 0x70ed9bade13bc7b7, 0x83467e7a0e0db5f, 0x2edb100f286c3cc2 } },
                 { { 0xe786890dc3e92812, 0xaa840f633c4d0061, 0xd73779dc2e753e05, 0xb0cf45d93b45890 },
                   { 0x1dc2a49708e5d8f, 0x189d394aac19ff2e, 0x6c365b59d178d821, 0x4a15ee554b55690 } } } };
    fq12 expected = { { { { 0x9a56f1e63b1f0db8, 0xd629a6c847f6cedd, 0x4a179c053a91458b, 0xa84c02b0b6d7470 },
                          { 0x3c7caa982d46f3a6, 0x2ce9b0c20c31e93a, 0x2eba2128e00999be, 0x2f509145620d470c } },
                        { { 0x8d1840c0a5e1beb1, 0x138909972699551, 0x6a2e46c29fc2e5e1, 0x2d5cc6154756f1f4 },
                          { 0x5c8d20bf46b48f23, 0x6f2cfc3bbdc5a317, 0xc528bbde6b995609, 0x24d37fc007b3c428 } },
                        { { 0xe1eea948d237426c, 0xe5c87a399cc972b1, 0x9747645c534da08a, 0x1e805d835ba889b4 },
                          { 0xdca7dd1573c013f7, 0x49a6b1b9c1877fe4, 0x352a43009f945986, 0x1136516f2fe27f7e } } },
                      { { { 0xf1591bdf82b9d577, 0x1ecfa28b5fe484a1, 0xe0e909e81b7b33de, 0x9a74f7ffc66975f },
                          { 0x9014340deeb7e8e0, 0xdf43f8e4e9470acf, 0x28e60662f1312e80, 0x17aadacec0d56c71 } },
                        { { 0xdb21cdcd8b4802fa, 0xb576311ef4863621, 0x28198e0757da3a32, 0x792df3cde67eb9a },
                          { 0x373dde8dfb6dceb3, 0xa0feac44ec583fb4, 0x257146bc7ad7d5c2, 0x1ee0a5c45a91938b } },
                        { { 0xf8c975188dd668a5, 0xfa38a6144e0c5451, 0x8ebdddc91016c224, 0x13fe7e09fe48aefb },
                          { 0x2ce375ffd1c12d33, 0xc2099e064cd9724d, 0x9c54b742a4d8bd59, 0x1c79d60ac5202c8c } } } };
    fq12 result = a.frobenius_map_three();
    EXPECT_EQ(result, expected);
}

TEST(fq12, frobenius_map_two)
{
    fq12 a = { { { { 0x52c2cc6e77bfe9bb, 0xd03d98cc3fd6d95, 0xfaeb6d6577aa9a30, 0x1ea38b81330e34df },
                   { 0x1f55d493000a14f3, 0x1db7ec50e2f5a356, 0xf3cfcc74b91481ae, 0x256fe76342b33dbb } },
                 { { 0xf3e95f622620a0f9, 0xe297badf08d73c22, 0x4df25d06ae059cfb, 0x16db699bc5bbddcb },
                   { 0xac821bca1b523880, 0x16e1c766941d5b3d, 0xcb1c3f0728eead67, 0x6a5bd44c11dc548 } },
                 { { 0xb6f165cba1492db7, 0x4b8dccf49d8ecb53, 0x4e9a8c3a5c91689f, 0x20ecb1ddd8c9db20 },
                   { 0xe7564097b01c2415, 0xf476520c5e9db5e6, 0xfe0ceb51798a245a, 0x2e9b3ec7fb7ad207 } } },
               { { { 0xf6e41a9b92434e0f, 0xeac46c17d2196da2, 0x44ac37aaba7d0518, 0x180b934a2302bc95 },
                   { 0xe2f112f1202f2a60, 0xdc6b42ce5b35837a, 0xfcee96f99e45e6b, 0x16f32ea5beadafe6 } },
                 { { 0xf11a764f75cd1ed6, 0x5b8605f1e9098788, 0xea81f341743177be, 0x1cafbefd2f6c5fa2 },
                   { 0x17bee2a7f91295fe, 0xb6b9716d19e4bbbc, 0x2f2550ee4c7ac30, 0x262edd6f32297cd5 } },
                 { { 0x430e641e6e94258d, 0xb1755bba0763e432, 0x3db56777846ee870, 0x22afd677233812e3 },
                   { 0xf89461bd1d9c3fa3, 0x9e25f21b44ad86a4, 0xfd1ed29f62168344, 0x1b337ab64bd9b0af } } } };
    fq12 expected = { { { { 0x52c2cc6e77bfe9bb, 0xd03d98cc3fd6d95, 0xfaeb6d6577aa9a30, 0x1ea38b81330e34df },
                          { 0x1f55d493000a14f3, 0x1db7ec50e2f5a356, 0xf3cfcc74b91481ae, 0x256fe76342b33dbb } },
                        { { 0x4d9bb69ca210d241, 0x42463a9de9a1298b, 0x6366c00ae6b366b2, 0x285cb81559b3c407 },
                          { 0x36f8a22587e7744b, 0xcba6fb2211505810, 0x357b875177bf4b97, 0x2da32e219de1632e } },
                        { { 0xb924dcf76fafea68, 0x430e02ed7ea41521, 0x29f4e4fcd758836c, 0x4839f24c2b447d },
                          { 0x7132411924f9b60d, 0xdd3d6a51478988d7, 0x98540139abbbd312, 0xc1409eeafe2aac } } },
                      { { { 0x222382f8f84c512c, 0x4b3f1d0eb307d6fd, 0x22f8fc60fe916c14, 0x1e63f42d2acbd109 },
                          { 0xf82d6103c6305cf4, 0x804b64f5caf58e7b, 0x69b8b37fa1383c9b, 0x2e74c0b9738bac8a } },
                        { { 0x4b0615c762afde71, 0x3bfb649f7f684304, 0xcdce52750d4fe09f, 0x13b48f75b1c54086 },
                          { 0x2461a96edf6a6749, 0xe0c7f9244e8d0ed1, 0xb55df0a79cb9ac2c, 0xa357103af082354 } },
                        { { 0xe1148c424a589341, 0x40ab0d25fb7fd0d1, 0x7909a54a9569db90, 0x99bde98bbc4352f },
                          { 0xfaa4fdcf224e38ee, 0x42b25f170bf5f577, 0xc13bf097c75be619, 0xbcb9923cbd60387 } } } };
    fq12 result = a.frobenius_map_two();
    EXPECT_EQ(result, expected);
}

TEST(fq12, frobenius_map_one)
{
    fq12 a = { { { { 0x6c9edca7f0d6f6e, 0x7bb482de96b01e0, 0xb04fc4b2b2ea7e6, 0x4d9efc00ceb8323 },
                   { 0xb55c2222935ee583, 0x9c114ab89499b4da, 0x771cb5cabe1f458a, 0x1c3f0ac5303a5935 } },
                 { { 0x524feabf94af29ea, 0x95573536ab8b6ced, 0x524e16790930912c, 0x280d5af94a3424d0 },
                   { 0xb6bdb5285238031c, 0x961e21ab4b9f7945, 0xa99257ecdc41179e, 0x25e9db7f50b8546f } },
                 { { 0x8ca5a9882dc185c4, 0xa497430d9ad2eebf, 0x29ed717c08faa305, 0xda59ed41c4283dc },
                   { 0x175ebf044916d79d, 0x3ed791b6263d56f1, 0x5c7c8932a433f839, 0x285eee6d768929ed } } },
               { { { 0x9195748c97fc7d6b, 0xa1da2592e5bde3bb, 0xe5f0358e5d6cd5b, 0xb890130084b6b73 },
                   { 0xb50d2c5ff65b68cd, 0xccefdc002bc84549, 0x732d99161ec379d5, 0x2c722b5ccbe40e2c } },
                 { { 0x1a3eaa24332c6fec, 0x8071b2dfdcbd55b9, 0x8fe8eb04d17c636a, 0x19a62bd610f3804d },
                   { 0x5e3a6b8238a14511, 0x70ef5efffc4e9e0e, 0x4e9b9a99caea296d, 0x28b9c9b70b4a747 } },
                 { { 0xc8f5f5664e3f17b4, 0x9e7b5d54d9e15481, 0xbd988b509f7d50b3, 0x101e343da280a34d },
                   { 0xbbb066284977a03, 0x264fc527ee8e520b, 0xacfa9508d9838c79, 0x191aa234984e211d } } } };
    fq12 expected = { { { { 0x6c9edca7f0d6f6e, 0x7bb482de96b01e0, 0xb04fc4b2b2ea7e6, 0x4d9efc00ceb8323 },
                          { 0x86c469f4451e17c4, 0xfb701fd8d3d815b2, 0x41338febc36212d2, 0x142543adb0f746f4 } },
                        { { 0xf4f695f59ce42cc7, 0xfab5aeca9715cd7, 0x8b69eaaf2cd76201, 0x364198e07630ca1 },
                          { 0x3cfc0f380cf3ba4a, 0xa1b605c52d334134, 0x321851c92680ca6c, 0x1749c78adebf8a5 } },
                        { { 0xeb71d65e04f06a8e, 0x569282ba59fbff0c, 0x1fb36eb4c4a1775, 0x22e79a74ea6bc0e6 },
                          { 0xe14b43fe62621be8, 0x36134c26ff43c3be, 0x3bd5b11835a8d7ee, 0x2e3b0f601d37b2c9 } } },
                      { { { 0xbda31ec838b3068d, 0x4f5f85130ea53c9c, 0xdae0c5f1f50979a1, 0x22eb05e4599b8f58 },
                          { 0xac23aa05132e266c, 0x5f0aa178c3a8f897, 0xe63225d0708133b9, 0xeeed4318f7539dd } },
                        { { 0x4249d30a2f88f55d, 0x10a7f2448ad57e4f, 0x260e76b439322dff, 0x1cb7e78896fd543c },
                          { 0x6602e7e93a714d67, 0x7398f14acf72c7e0, 0x8028d203d5e4928, 0x7d1fad57418b580 } },
                        { { 0xcba1922169de670, 0xcd20689212638b5e, 0x8dbbc53af7639bbb, 0x57a19a043d38c39 },
                          { 0x2b2d3090bfb1118b, 0xa752e789e316e0c7, 0xc1c4d33385bc3e10, 0x2610936b5468ba45 } } } };
    fq12 result = a.frobenius_map_one();
    EXPECT_EQ(result, expected);
}

TEST(fq12, to_montgomery_form)
{
    fq12 result = fq12::zero();
    result.c0.c0.c0.data[0] = 1;
    fq12 expected = fq12::one();
    result = result.to_montgomery_form();
    EXPECT_EQ(result, expected);
}

TEST(fq12, from_montgomery_form)
{
    fq12 result = fq12::one();
    fq12 expected = fq12::zero();
    expected.c0.c0.c0.data[0] = 1;
    result = result.from_montgomery_form();
    EXPECT_EQ(result, expected);
}

TEST(fq12, mul_sqr_consistency)
{
    fq12 a = fq12::random_element();
    fq12 b = fq12::random_element();
    fq12 t1 = a - b;
    fq12 t2 = a + b;
    fq12 mul_result = t1 * t2;
    fq12 sqr_result = a.sqr() - b.sqr();
    EXPECT_EQ(mul_result, sqr_result);
}

TEST(fq12, add_mul_consistency)
{
    fq12 multiplicand = fq12::zero();
    multiplicand.c0.c0.c0.data[0] = 9;
    multiplicand = multiplicand.to_montgomery_form();

    fq12 a = fq12::random_element();
    fq12 result = a + a;
    result += result;
    result += result;
    result += a;

    fq12 expected = a * multiplicand;

    EXPECT_EQ(result, expected);
}

TEST(fq12, sub_mul_consistency)
{
    fq12 multiplicand = fq12::zero();
    multiplicand.c0.c0.c0.data[0] = 5;
    multiplicand = multiplicand.to_montgomery_form();

    fq12 a = fq12::random_element();
    fq12 result = a + a;
    result += result;
    result += result;
    result -= a;
    result -= a;
    result -= a;

    fq12 expected = a * multiplicand;
    EXPECT_EQ(result, expected);
}
