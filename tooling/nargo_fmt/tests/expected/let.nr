//@error_on_lost_comment=false
fn let_() {
    let fn_call = my_function(
        some_function(10, "arg1", another_function()),
        another_func(20, some_function(), 30),
    );
    let array = [[[1, 2, 3], [4, 5, 6]], [[7, 8, 9], [10, 11, 12]], [[13, 14, 15], [16, 17, 18]]];

    let padded_sha256_hash: [u8; 259] = [
        // Padded hash
        209, 50, 135, 178, 4, 155, 190, 229, 228, 111, 61, 174, 8, 49, 48, 116, 90, 226, 77, 7, 111,
        27, 19, 113, 154, 48, 138, 136, 138, 15, 230, 132, 32, 4, 0, 5, 1, 2, 4, 3, 101, 1, 72, 134,
        96, 9, 6, 13, 48, 49, 48, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1, 0,
        // Rest is padded with 0s until max bytes
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    let a = BigUint56 {
        limbs: [
            1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ],
    };

    let person = Person {
        first_name: "John",
        last_name: "Doe",
        home_address: Address { street: "123 Main St", city: "Exampleville", zip_code: "12345" },
    };

    let person = Person {
        first_name: "John",
        last_name: "Doe",
        home_address: Address {
            street: "123 Main St",
            city: "Exampleville",
            zip_code: "12345",
            master: Person {
                first_name: "John",
                last_name: "Doe",
                home_address: Address {
                    street: "123 Main St",
                    city: "Exampleville",
                    zip_code: "12345",
                },
            },
        },
    };

    let expr = MyExpr {
        // A boolean literal (true, false).
        kind: ExprKind::Bool(true),
    };

    let expr = MyExpr { /*A boolean literal (true, false).*/ kind: ExprKind::Bool(true) };

    let mut V = crate2::MyStruct { Q: x };
    let mut V = crate2::MyStruct {};
    let mut V = crate2::MyStruct { /*test*/ };
    let mut V = crate2::MyStruct {
        // sad
    };
}
