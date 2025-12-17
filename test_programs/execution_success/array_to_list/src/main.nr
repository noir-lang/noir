// Converts an array into a list.
fn as_list_push<T, let N: u32>(xs: [T; N]) -> [T] {
    let mut list = &[];
    for elem in xs {
        list = list.push_back(elem);
    }
    list
}

// Expected that x == 0 and y == 1
fn main(x: u32, y: pub u32) {
    let xs: [Field; 0] = [];
    let ys: [Field; 1] = [1];
    let zs: [Field; 2] = [1, 2];
    let ws: [Field; 3] = [1; 3];
    let qs: [Field; 4] = [3, 2, 1, 0];

    let mut dynamic: [Field; 4] = [3, 2, 1, 0];
    let dynamic_expected: [Field; 4] = [1000, 2, 1, 0];
    dynamic[x] = 1000;

    assert(x != y);
    assert(xs.as_list() == as_list_push(xs));
    assert(ys.as_list() == as_list_push(ys));
    assert(zs.as_list() == as_list_push(zs));
    assert(ws.as_list() == as_list_push(ws));
    assert(qs.as_list() == as_list_push(qs));

    assert(dynamic.as_list()[0] == dynamic_expected[0]);
    assert(dynamic.as_list()[1] == dynamic_expected[1]);
    assert(dynamic.as_list()[2] == dynamic_expected[2]);
    assert(dynamic.as_list()[3] == dynamic_expected[3]);
    assert(dynamic.as_list().len() == 4);

    regression_4609_append_lists(x, y);
    regression_4609_append_dynamic_lists(x, y);
}

fn regression_4609_append_lists(x: u32, y: u32) {
    let sl = [x, 1, 2, 3].as_list();
    let sl2 = [y, 5, 6].as_list();
    let sl3 = sl.append(sl2);
    assert(sl3[0] == x);
    assert(sl3[4] == y);
}

fn regression_4609_append_dynamic_lists(x: u32, y: u32) {
    let mut sl = [x, 1, 2, 3].as_list();
    sl[x] = x + 10;
    let mut sl2 = [y, 5, 6].as_list();
    sl2[y] = y + 5;
    let sl3 = sl.append(sl2);
    assert(sl3[0] == 10);
    assert(sl3[4] == y);
    assert(sl3[5] == 6);
}
