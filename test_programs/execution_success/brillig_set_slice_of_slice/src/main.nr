struct Property
{
    key : [u8],
    value : [u8],
}

struct JSON
{
    doc : [Property]
}

unconstrained fn slice_eq(self: [u8], other: [u8]) -> bool {
    let mut equal = true;
    for i in 0..self.len() {
        if self[i] != other[i] {
            equal = false;
        }
    }
    equal
}

// This test acts a regression for issue #3476
unconstrained fn main() {
    let mut json = JSON { doc: [] };
    let mut prop = Property { key: [], value:[] };

    let other_prop = Property { key: [0, 1, 2], value:[10] };
    json.doc = json.doc.push_back(other_prop);

    for i in 0..3 {
        prop.key = prop.key.push_back(i as u8);
    }
    prop.value = prop.value.push_back(5);

    // add property to json or replace existing
    let len : Field = json.doc.len();
    let mut found = false;
    for i in 0..len 
    { 
        if (!found) 
        { 
            if (slice_eq(prop.key, json.doc[i].key))
            { 
                json.doc[i].value = prop.value; 
                found = true; 
            } 
        } 
    }
    assert(found == true);
    assert(json.doc[0].value[0] == 5);
}