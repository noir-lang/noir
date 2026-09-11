use std::path::Path;

use fm::FileManager;
use noirc_driver::{CrateId, file_manager_with_stdlib};
use noirc_frontend::hir::{Context, FunctionNameMatch, ParsedFiles, def_map::TestFunction};

use crate::{insert_all_files_for_workspace_into_file_manager, parse_all, workspace::Workspace};

pub struct TestDefinition {
    pub name: String,
    pub function: TestFunction,
}
pub fn get_test_function_for_debug(
    crate_id: CrateId,
    context: &Context,
    test_name: &str,
) -> Result<TestDefinition, String> {
    let test_pattern = FunctionNameMatch::Contains(vec![test_name.into()]);

    let test_functions = context.get_all_test_functions_in_crate_matching(&crate_id, &test_pattern);

    let (test_name, test_function) = match test_functions {
        matchings if matchings.is_empty() => {
            return Err(format!("`{test_name}` does not match with any test function"));
        }
        matchings if matchings.len() == 1 => matchings.into_iter().next().unwrap(),
        matchings => {
            let exact_match_op = matchings
                .into_iter()
                .filter(|(name, _)| name.split("::").last() == Some(test_name))
                .collect::<Vec<(String, TestFunction)>>();
            // There can be multiple matches but only one that matches exactly
            // this would be the case of tests names that englobe others
            // i.e.:
            //  - test_something
            //  - unconstrained_test_something
            // in this case, looking up "test_something" throws two matchings
            // but only one matches exact
            if exact_match_op.len() == 1 {
                exact_match_op.into_iter().next().unwrap()
            } else {
                return Err(format!("`{test_name}` matches with more than one test function"));
            }
        }
    };

    let test_function_has_arguments =
        !context.def_interner.function_meta(&test_function.id).parameters.is_empty();

    if test_function_has_arguments {
        return Err(String::from("Cannot debug tests with arguments"));
    }
    Ok(TestDefinition { name: test_name, function: test_function })
}

pub fn load_workspace_files(workspace: &Workspace) -> (FileManager, ParsedFiles) {
    let mut file_manager = file_manager_with_stdlib(Path::new(""));
    insert_all_files_for_workspace_into_file_manager(workspace, &mut file_manager);

    let parsed_files = parse_all(&file_manager);
    (file_manager, parsed_files)
}
