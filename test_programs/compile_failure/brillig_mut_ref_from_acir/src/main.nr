use dep::std::field::bn254::assert_gt;

struct ListItem {
    key: Field,
    previous: Field,
    next: Field,
}

impl ListItem {
    fn default() -> ListItem {
        ListItem {
            key: 0,
            previous: 0,
            next: 0,
        }
    }
}

struct Map<Size> {
    entries: [ListItem; Size],
    size: Field,
    is_empty: bool,
    first_index: Field,
    last_index: Field,
}

unconstrained fn find_previous_key_location_crash<Size>(map: &mut Map<Size>, key: Field) -> (Field) {
    let mut found_index: Field = 0;
    found_index
}

impl<Size> Map<Size> {
    fn default() -> Map<Size> {
        Map{
            entries: [ListItem::default(); Size], // todo fix
            size: 0,
            is_empty: true,
            first_index: 0,
            last_index: 0,
        }
    }


    fn insert_crashy(&mut self, key: Field) {
        let previous_index = find_previous_key_location_crash(self, key);
        self.entries[previous_index].key = key;
    }

}

fn main(x: Field, y: pub Field) {
    let mut test_list: Map<10> = Map::default();
    test_list.insert_crashy(x);
    test_list.insert_crashy(x + y);
    test_list.insert_crashy(x + y * y);
}