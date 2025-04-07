
    fn main() {
        let mut bar = Bar {};
        let _ = bar.bar();
    }

    struct Bar {}

    impl Bar {
        fn bar(&mut self) {
            let _ = self;
        }
    }
    