use dep::std;
struct Item {
    id: Field,
}

impl Item {
    fn log(self) {
        let id = self.id;
        std::println(id);
    }
}

fn create<V>(something: V) -> V {
    something
}

fn main() {
    let a = Item { id: 1 };
    let b = create(a);
    let _id = b.id; 
    // Regression for:  cannot find this method
    b.log();
}
