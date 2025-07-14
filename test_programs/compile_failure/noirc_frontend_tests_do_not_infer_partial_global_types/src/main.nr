
        pub global ARRAY: [Field; _] = [0; 3];
        pub global NESTED_ARRAY: [[Field; _]; 3] = [[]; 3];
        pub global STR: str<_> = "hi";
                 
        pub global NESTED_STR: [str<_>] = &["hi"];
        pub global FORMATTED_VALUE: str<5> = "there";
        pub global FMT_STR: fmtstr<_, _> = f"hi {FORMATTED_VALUE}";
        pub global TUPLE_WITH_MULTIPLE: ([str<_>], [[Field; _]; 3]) = 
            (&["hi"], [[]; 3]);

        fn main() { }
    