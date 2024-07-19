use serde_json::Value;
use std::env;

mod read_write_json;
use read_write_json::{save_to_file, serialize_file};
mod prettify;
use prettify::correct_path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        panic!("Arguments must be exactly two. First the src file which must be formatted and the second one should be the destination file name");
    }

    let src_filename = args[1].clone();
    let dest_filename = args[2].clone();
    let ser_json: Value = serialize_file(src_filename);

    let prettified_json: String = prettify::prettify_value(ser_json, "", false);
    let final_pretty_json: String = correct_path(&prettified_json);

    save_to_file(dest_filename, final_pretty_json);
}
