use itertools::Itertools;

/// Get Relations Imports
///
/// We may have multiple relation files in the generated foler
/// This method will return all of the imports for the relation header files
pub fn get_relations_imports(name: &str, relations: &[String], permutations: &[String]) -> String {
    let all_relations = flatten(&[relations.to_vec(), permutations.to_vec()]);
    let transformation = |relation_name: &_| {
        format!("#include \"barretenberg/relations/generated/{name}/{relation_name}.hpp\"")
    };

    map_with_newline(&all_relations, transformation)
}

/// Sanitize Names
///
/// Column titles that we get from pil contain . to distinguish which pil namespace they belong to
/// We need to replace these with _ to make them valid C++ identifiers
pub fn sanitize_name(string: &str) -> String {
    string.replace(['.', '[', ']'], "_")
}

/// Capitalize
pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// Map With Newline
/// This utility function is used all over the codegen pipeline
/// It takes a list, usually the names of columns in an execution trace and applies a string transformation "op"
/// to each element in the list
pub fn map_with_newline<Func>(list: &[String], op: Func) -> String
where
    Func: Fn(&String) -> String,
{
    transform_map(list, op).join("\n")
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

/// Create Forward As Tuple
///
/// Helper function to create a forward as tuple cpp statement
pub fn create_forward_as_tuple(settings: &[String]) -> String {
    let adjusted = settings.iter().map(|col| format!("in.{col}")).join(",\n");
    format!(
        "
        return std::forward_as_tuple(
            {}
        );
    ",
        adjusted
    )
}

// TODO: may make sense to move the below around a bit
pub fn create_get_const_entities(settings: &[String]) -> String {
    let forward = create_forward_as_tuple(settings);
    format!(
        "
    template <typename AllEntities> static inline auto get_const_entities(const AllEntities& in) {{
        {forward}
    }}
    "
    )
}

pub fn create_get_nonconst_entities(settings: &[String]) -> String {
    let forward = create_forward_as_tuple(settings);
    format!(
        "
    template <typename AllEntities> static inline auto get_nonconst_entities(AllEntities& in) {{
        {forward}
    }}
    "
    )
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
