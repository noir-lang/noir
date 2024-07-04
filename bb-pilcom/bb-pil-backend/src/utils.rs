/// Sanitize Names
///
/// Column titles that we get from pil contain . to distinguish which pil namespace they belong to
/// We need to replace these with _ to make them valid C++ identifiers
pub fn sanitize_name(string: &str) -> String {
    string.replace(['.', '[', ']'], "_")
}

/// Collect Col
///
/// Transforms columns from powdr representation ( where the witnesses are linked )
/// Into a version where we just keep the columns
/// As this is all we are about
pub fn collect_col<Func>(list: &[String], op: Func) -> Vec<String>
where
    Func: Fn(&String) -> String,
{
    list.iter().map(op).collect::<Vec<String>>()
}

/// Transform Map
///
/// Apply a transformation to a list of strings
pub fn transform_map<Func>(list: &[String], op: Func) -> Vec<String>
where
    Func: Fn(&String) -> String,
{
    list.iter().map(op).collect::<Vec<String>>()
}

/// Flatten
///
/// Returns a flattened concatenation of the input arrays
pub fn flatten(list: &[Vec<String>]) -> Vec<String> {
    let arr = list.iter().cloned();
    arr.into_iter().flatten().collect()
}

/// Snake Case
///
/// Transform camel case string into snake case, such as: RedFlower --> red_flower
pub fn snake_case(input: &str) -> String {
    let mut result = String::new();

    // Handle the first character
    if input.is_empty() {
        return result; // Empty input
    }
    let mut first_char = input.chars().next().unwrap();
    if first_char.is_uppercase() {
        first_char = first_char.to_ascii_lowercase();
    }
    result.push(first_char);

    // Process remaining characters
    for ch in input.chars().skip(1) {
        if ch.is_uppercase() {
            result.push('_');
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push(ch);
        }
    }

    result
}

pub fn sort_cols(cols: &[String]) -> Vec<String> {
    let mut cols = cols.to_vec();
    cols.sort();
    cols
}
