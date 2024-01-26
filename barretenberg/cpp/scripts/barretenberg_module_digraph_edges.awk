# Function to extract words between parentheses
function extract_edges(line) {
    match(line, /\(.*\)/);  # Find the portion within parentheses
    line = substr(line, RSTART + 1, RLENGTH - 2);  # Extract the words
    gsub(/^[ ]+/, "", line); # Remove leading spaces and tabs
    gsub(/[ ]+$/, "", line); # Remove trailing spaces and tabs
    gsub(/[ ]+/, " ", line); # Sub multiple spaces for a single space
    split(line, modules, " "); # Split into an array of words

    # If node has no dependencies, just add the node
    if (length(modules)==1) {
        print modules[1];
    }
    else { # add edges
        for (i = 2; i <= length(modules); i++) {
            print modules[1]" -> "modules[i];
        }
    }
}

# Main AWK script
{
    # Concatenate lines if the opening parenthesis is not closed
    while (!/\)/) {
        current_line = $0;
        getline;
        $0 = current_line $0;
    }

    # Check if the line begins with "barretenberg_module". If so, extact the digraph edges
     function_name = "barretenberg_module";
    if ($0 ~ "^" function_name "\\(") {
        extract_edges($0);
    }
}
