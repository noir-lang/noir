use dep::protocol_types::point::Point;

// A set of generators generated with `derive_generators(...)` function from noir::std
global Ga1 = Point { x: 0x30426e64aee30e998c13c8ceecda3a77807dbead52bc2f3bf0eae851b4b710c1, y: 0x113156a068f603023240c96b4da5474667db3b8711c521c748212a15bc034ea6, is_infinite: false };
global Ga2 = Point { x: 0x2825c79cc6a5cbbeef7d6a8f1b6a12b312aa338440aefeb4396148c89147c049, y: 0x129bfd1da54b7062d6b544e7e36b90736350f6fba01228c41c72099509f5701e, is_infinite: false };
global Ga3 = Point { x: 0x0edb1e293c3ce91bfc04e3ceaa50d2c541fa9d091c72eb403efb1cfa2cb3357f, y: 0x1341d675fa030ece3113ad53ca34fd13b19b6e9762046734f414824c4d6ade35, is_infinite: false };

mod test {
    use crate::generators::{Ga1, Ga2, Ga3};
    use dep::protocol_types::point::Point;
    use std::hash::derive_generators;

    #[test]
fn test_generators() {
        let generators: [Point; 3] = derive_generators("aztec_nr_generators".as_bytes(), 0);
        assert_eq(generators[0], Ga1);
        assert_eq(generators[1], Ga2);
        assert_eq(generators[2], Ga3);
    }
}
