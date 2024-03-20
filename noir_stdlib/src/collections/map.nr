use crate::field::bn254::assert_gt;

struct ListItem {
    key: Field,
    value: Field,
    previous: Field,
    next: Field,
}

impl ListItem {
    fn default() -> ListItem {
        ListItem {
            key: 0,
            value: 0,
            previous: 0,
            next: 0,
        }
    }
}

struct HashMap<Size> {
    entries: [ListItem; Size],
    size: Field,
    is_empty: bool,
    first_index: Field,
    last_index: Field,
}

impl<Size> HashMap<Size> {
    pub fn default() -> HashMap<Size> {
        HashMap{
            entries: [ListItem::default(); Size], // todo fix
            size: 0,
            is_empty: true,
            first_index: 0,
            last_index: 0,
        }
    }

    unconstrained fn check_for_collision(self, key: Field) -> (Field, bool) {
        let mut found_index: Field = 0;
        let mut found: bool = false;
        for i in 0 .. self.size as u64 {
            if (self.entries[i].key == key)
            {
                found_index = i as Field;
                found = true;
            }
        }
        (found_index, found)
    }

    unconstrained fn find_previous_key_location(self, key: Field) -> (Field, bool, bool, bool) {
        let mut found_index: Field = 0;
        let mut insert_between_two_entries: bool = false;
        let mut insert_at_start: bool = false;
        let mut insert_at_end: bool = false;

        if (key.lt(self.entries[self.first_index].key) & !self.is_empty)
        {
            found_index = self.first_index;
            insert_at_start = true;
        }
        else if (self.entries[self.last_index].key.lt(key) & !self.is_empty)
        {
            found_index = self.last_index;
            insert_at_end = true;
        }

        for i in 0 .. self.size as u64 {
            let previous_index = self.entries[i].previous;
            let previous_item = self.entries[previous_index].key;
            if (key.lt(self.entries[i].key) & previous_item.lt(key))
            {
                found_index = previous_index as Field;
                insert_between_two_entries = true;
            }
        }
        (found_index, insert_between_two_entries, insert_at_start, insert_at_end)
    }

    pub fn insert(&mut self, key: Field, value: Field) {
        // TODO: make the check that Size < 2^16 an unconstrained compile-time check
        (Size - self.size).assert_max_bit_size(16);
        let (previous_index, insert_between_two_entries, insert_at_start, insert_at_end) = self.find_previous_key_location(key);
        let (collision_index, found_collision) = self.check_for_collision(key);
        let is_first_entry = self.is_empty;

        // Assert that one (and only one) of is_first_entry, insert_at_start, insert_at_end, insert_between_two_entries, found_collision is true
        let path_check = is_first_entry as Field + insert_at_start as Field + insert_at_end as Field + insert_between_two_entries as Field + found_collision as Field;
        assert_eq(path_check, 1, f"path_check is not 1, is_first_entry = {is_first_entry}, insert_at_start = {insert_at_start}, insert_at_end = {insert_at_end}, insert_between_two_entries = {insert_between_two_entries}, found_collision = {found_collision}");

  
        let next_index = self.entries[previous_index].next;
        let previous = self.entries[previous_index].key;
        let next = self.entries[next_index].key;

        // We apply two greater-than checks.
        // Case 1: We insert in between two existing entries
        // key > previous
        // next > key
        // Case 2: We insert at start of list
        // next > key
        // Case 3: We insert at end of list
        // key > previous
        // Case 4: Collision!
        // key == next
        // Case 5: List is empty

        // key > previous check
        let apply_key_gt_previous_check: bool = insert_between_two_entries | insert_at_end;
        let apply_next_gt_key_check: bool = insert_between_two_entries | insert_at_start;

        let key_lhs = if apply_key_gt_previous_check { key } else { 1 };
        let previous_rhs = if apply_key_gt_previous_check { previous } else { 0 };
        assert_gt(key_lhs, previous_rhs);

        let next_lhs = if apply_next_gt_key_check { next } else { 1 };
        let key_rhs = if apply_next_gt_key_check { key } else { 0 };
        assert_gt(next_lhs, key_rhs);


        // If we have collided, validate previous == key
        if (found_collision)
        {
            assert_eq(previous, key);
        }

        // If insert_at_start, validate self.entries[previous_index].previous = invalid index
        if (insert_at_start)
        {
            assert_eq(previous_index, self.first_index);
            self.first_index = self.size;
        }

        // If insert_at_end, validate self.entries[previous_index].next = invalid index
        if (insert_at_end)
        {
            assert_eq(previous_index, self.last_index);
            self.last_index = self.size;
        }
    
        if (self.is_empty)
        {
            self.first_index = self.size;
            self.last_index = self.size;
        }

        // New entry.
        // If insert_at_end OR first entry, next = -1, else next = next_index
        let new_item_next = if (insert_at_end | is_first_entry) { Size - 1 } else { next_index };
        // If insert_at_start OR first entry, previous = -1, else previous = previous_index
        let new_item_previous = if (insert_at_start | is_first_entry) { Size -1 } else { previous_index };

        // we DONT update previous index if: first entry OR collision OR insert at start
        let update_previous_index = insert_at_end | insert_between_two_entries;
        self.entries[previous_index].next = if update_previous_index { self.size } else { self.entries[previous_index].next};

        // we DONT update next index if: first entry OR collision OR insert at end
        let update_next_index = insert_at_start | insert_between_two_entries;
        self.entries[next_index].previous = if update_next_index { self.size } else { self.entries[next_index].previous };

        let new_entry_index = if found_collision { collision_index } else { self.size };
        self.entries[new_entry_index] = ListItem{ key: key, value: value, previous: new_item_previous, next: new_item_next };

        self.size += 1 - found_collision as Field;
        self.is_empty = false;
    }

    unconstrained fn find_key_location(self, key: Field) -> u8 {
        let mut found: bool = false;
        let mut index: u8 = 0;
        for i in 0..Size {
            if (key == self.entries[i].key) {
                index = i;
                found = true;
            }
        }
        assert(found == true);
        index
    }


    pub fn at(self, key: Field) -> Field {
        let index: u8 = self.find_key_location(key);
        assert(self.entries[index].key == key);
        self.entries[index].value
    }


    pub fn get(self, key: Field) -> ListItem {
        let index: u8 = self.find_key_location(key);
        assert(self.entries[index].key == key);
        self.entries[index]
    }
}

#[test]
fn test_insert() {
    let mut test_list: HashMap<5> = HashMap::default();
    test_list.insert(123, 456);
    assert(test_list.size == 1);
    let mut result = test_list.at(123);
    assert(result == 456);

    test_list.insert(128, 999);
    assert(test_list.size == 2);
    result = test_list.at(128);
    assert(result == 999);

    let first = test_list.get(123);
    let second = test_list.get(128);
    assert(test_list.entries[first.next].key == second.key);
    assert(test_list.entries[second.previous].key == first.key);
    assert(first.next == 1);
    assert(second.previous == 0);
    assert(first.previous == 4);
    assert(second.next == 4);

    test_list.insert(127, 333);
    assert(test_list.size == 3);
    result = test_list.at(127);
    assert(result == 333);
    test_list.insert(123, 457);
    assert(test_list.size == 3);
    result = test_list.at(123);
    assert(result == 457);

    test_list.insert(1, 3);
    assert(test_list.size == 4);
    result = test_list.at(1);
    assert(result == 3);
}
