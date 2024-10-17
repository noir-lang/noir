use anyhow::{ensure, Result};
use toml_edit::{DocumentMut, Item, Table};

pub fn get_table_mut<'a>(doc: &'a mut DocumentMut, path: &[&str]) -> Result<&'a mut Item> {
    return visit(doc.as_item_mut(), path);

    fn visit<'a>(item: &'a mut Item, path: &[&str]) -> Result<&'a mut Item> {
        if let Some(segment) = path.first() {
            let item = item[segment].or_insert({
                let mut table = Table::new();
                table.set_implicit(true);
                Item::Table(table)
            });

            ensure!(
                item.is_table_like(),
                "the table `{segment}` could not be found."
            );
            visit(item, &path[1..])
        } else {
            assert!(item.is_table_like());
            Ok(item)
        }
    }
}
