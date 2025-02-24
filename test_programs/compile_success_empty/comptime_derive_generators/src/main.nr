use std::embedded_curve_ops::EmbeddedCurvePoint;

fn main() {
    comptime {
        // Result computed from executing `derive_generators` with non-comptime Noir
        let result = [
            EmbeddedCurvePoint {
                x: 0x0224a8abc6c8b8d50373d64cd2a1ab1567bf372b3b1f7b861d7f01257052d383,
                y: 0x2358629b90eafb299d6650a311e79914b0215eb0a790810b26da5a826726d711,
                is_infinite: false,
            },
            EmbeddedCurvePoint {
                x: 0x0f106f6d46bc904a5290542490b2f238775ff3c445b2f8f704c466655f460a2a,
                y: 0x29ab84d472f1d33f42fe09c47b8f7710f01920d6155250126731e486877bcf27,
                is_infinite: false,
            },
            EmbeddedCurvePoint {
                x: 0x0298f2e42249f0519c8a8abd91567ebe016e480f219b8c19461d6a595cc33696,
                y: 0x035bec4b8520a4ece27bd5aafabee3dfe1390d7439c419a8c55aceb207aac83b,
                is_infinite: false,
            },
            EmbeddedCurvePoint {
                x: 0x2c9628479de4181ea77e7b0913ccf41d2a74155b1d9c82eaa220c218781f6f3b,
                y: 0x278f86b8fd95520b5da23bee1a5e354dc5dcb0cb43d6b76e628ddbffb101d776,
                is_infinite: false,
            },
            EmbeddedCurvePoint {
                x: 0x0be1916f382e3532aa53a766fe74b1a983784caab90290aea7bf616bc371fb41,
                y: 0x0f65545005e896f14249956344faf9addd762b7573a487b58f805a361d920a20,
                is_infinite: false,
            },
            EmbeddedCurvePoint {
                x: 0x29ff8437ae5bec89981441b23036a22b7fd5bee9eff0e83c0dd5b87bfb5bd60e,
                y: 0x1fd247352b77e2676b22db23cf7cd482474f543e3480b5a39c42f839a306be10,
                is_infinite: false,
            },
            EmbeddedCurvePoint {
                x: 0x2f3bd4e98f8c8458cd58888749f0f5e582a43565767398e08e50e94b9b19a4d9,
                y: 0x1f534906d1aa8b4ba74ad9e3f85ae3f8295e51eaafd15b5d116801b96360205b,
                is_infinite: false,
            },
            EmbeddedCurvePoint {
                x: 0x27759098f425b76447c2c52728576803a1ac5de37bba875ac47cdcff539ab931,
                y: 0x0aa47ee64d12d856cfb81b595c1d60ceecb693f0fdae644746ff333e39f61db7,
                is_infinite: false,
            },
            EmbeddedCurvePoint {
                x: 0x015ca8d68616fde86c9108e3db04f588e0f308e60d367e963b7d460fe9a65e6c,
                y: 0x2cf918009dda942ac9d59903cd2d0294d8738f938b1394170d892a027d0f347b,
                is_infinite: false,
            },
            EmbeddedCurvePoint {
                x: 0x0d1783d5b256765515f3c9988df9f1ba7e6f5fb0248c8971fbc503ffd5187714,
                y: 0x2ebb434ff4857fc3621f3bc3c6b8002b17d02d9c204e75f19b8f0b99ea68402c,
                is_infinite: false,
            },
        ];

        let generators: [EmbeddedCurvePoint; 10] =
            std::hash::derive_generators("DEFAULT_DOMAIN_SEPARATOR".as_bytes(), 5);

        for i in 0..10 {
            assert(generators[i].x == result[i].x);
            assert(generators[i].y == result[i].y);
        }
    }
}
