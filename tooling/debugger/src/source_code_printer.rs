use acvm::acir::circuit::OpcodeLocation;
use codespan_reporting::files::Files;
use nargo::artifacts::debug::DebugArtifact;
use noirc_errors::Location;
use owo_colors::OwoColorize;
use std::ops::Range;

#[derive(Debug)]
enum PrintedLine<'a> {
    Skip,
    Ellipsis { number: usize },
    Content { number: usize, cursor: &'a str, content: &'a str, highlight: Option<Range<usize>> },
}

#[derive(Debug)]
struct PrintedLocation<'a> {
    location: Location,
    lines: Vec<PrintedLine<'a>>,
}

pub(crate) fn print_source_code_location(
    debug_artifact: &DebugArtifact,
    location: &OpcodeLocation,
) {
    let rendered_locations = render(debug_artifact, location);

    for loc in rendered_locations {
        print_location_path(debug_artifact, loc.location);

        for line in loc.lines {
            match line {
                PrintedLine::Skip => {}
                PrintedLine::Ellipsis { number } => {
                    println!("{:>3} {:2} {}", number.dimmed(), "", "...".dimmed(),);
                }
                PrintedLine::Content { number, cursor, content, highlight } => match highlight {
                    Some(highlight) => {
                        println!(
                            "{:>3} {:2} {}{}{}",
                            number,
                            cursor,
                            content[0..highlight.start].to_string().dimmed(),
                            &content[highlight.start..highlight.end],
                            content[highlight.end..].to_string().dimmed(),
                        );
                    }
                    None => {
                        println!(
                            "{:>3} {:2} {}",
                            number.dimmed(),
                            cursor.dimmed(),
                            content.to_string().dimmed(),
                        );
                    }
                },
            }
        }
    }
}

fn print_location_path(debug_artifact: &DebugArtifact, loc: Location) {
    let line_number = debug_artifact.location_line_number(loc).unwrap();
    let column_number = debug_artifact.location_column_number(loc).unwrap();

    println!("At {}:{line_number}:{column_number}", debug_artifact.name(loc.file).unwrap());
}

fn render<'a>(
    debug_artifact: &'a DebugArtifact,
    location: &OpcodeLocation,
) -> Vec<PrintedLocation<'a>> {
    let mut rendered_locations: Vec<PrintedLocation> = [].into();

    let locations = debug_artifact.debug_symbols[0].opcode_location(location);

    //TODO: use map instead?
    let Some(locations) = locations else { return rendered_locations };

    for loc in locations {
        let loc_line_index = debug_artifact.location_line_index(loc).unwrap();
        let loc_end_line_index = debug_artifact.location_end_line_index(loc).unwrap();

        // How many lines before or after the location's line we
        // print
        let context_lines = 5;

        let first_line_to_print =
            if loc_line_index < context_lines { 0 } else { loc_line_index - context_lines };

        let last_line_index = debug_artifact.last_line_index(loc).unwrap();

        // Print all lines that the current location spans
        let last_line_to_print = std::cmp::min(loc_end_line_index + context_lines, last_line_index);

        let source = debug_artifact.location_source_code(loc).unwrap();

        let lines = source
            .lines()
            .enumerate()
            .map(|(current_line_index, line)| {
                let current_line_number = current_line_index + 1;
                
                if current_line_index < first_line_to_print {
                    // Ignore lines before the context window we choose to show
                    PrintedLine::Skip
                } else if current_line_index == first_line_to_print && current_line_index > 0 {
                    // Denote that there's more lines before but we're not showing them
                    PrintedLine::Ellipsis { number: current_line_number }
                } else if current_line_index < loc_line_index {
                    // Print lines before the location start
                    // without highlighting
                    PrintedLine::Content {
                        number: current_line_number,
                        cursor: "",
                        content: line,
                        highlight: None,
                    }
                } else if current_line_index == loc_line_index {
                    let to_highlight = debug_artifact.location_in_line(loc).unwrap();

                    // Highlight current location from where it starts
                    // to the end of the current line
                    PrintedLine::Content {
                        number: current_line_number,
                        cursor: "->",
                        content: line,
                        highlight: Some(to_highlight),
                    }
                } else if current_line_index < loc_end_line_index {
                    // Highlight current line if it's contained by the current location
                    PrintedLine::Content {
                        number: current_line_number,
                        cursor: "",
                        content: line,
                        highlight: Some(Range { start: 0, end: line.len() - 1 }),
                    }
                } else if current_line_index == loc_end_line_index {
                    let to_highlight = debug_artifact.location_in_end_line(loc).unwrap();

                    // Highlight current location from the beginning
                    // of the line until the location's own end
                    PrintedLine::Content {
                        number: current_line_number,
                        cursor: "",
                        content: line,
                        highlight: Some(to_highlight),
                    }
                } else if current_line_index < last_line_to_print {
                    // Print lines after the location end
                    // without highlighting
                    PrintedLine::Content {
                        number: current_line_number,
                        cursor: "",
                        content: line,
                        highlight: None,
                    }
                } else if current_line_index == last_line_to_print && last_line_to_print < last_line_index {
                    // Denote that there's more lines after but we're not showing them,
                    // and stop printing
                    PrintedLine::Ellipsis { number: current_line_number }
                } else {
                    PrintedLine::Skip
                }
            })
            .collect();

        rendered_locations.push(PrintedLocation { location: loc, lines });
    }

    rendered_locations
}
