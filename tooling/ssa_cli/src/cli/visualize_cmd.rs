use std::fmt::Write;
use std::path::PathBuf;

use base64::Engine;
use clap::Args;
use color_eyre::eyre::{self, Context};
use noir_artifact_cli::commands::parse_and_normalize_path;
use noirc_evaluator::ssa::ssa_gen::Ssa;
use serde_json::json;

use crate::cli::write_output;

/// Parse the SSA and render the CFG for visualization with Mermaid.
#[derive(Debug, Clone, Args)]
pub(super) struct VisualizeCommand {
    /// Path to write the output to.
    ///
    /// If empty, the output will be written to stdout.
    #[clap(long, short, value_parser = parse_and_normalize_path)]
    pub output_path: Option<PathBuf>,

    /// Surround the output with brackets for direct embedding in a Markdown file.
    ///
    /// Useful for dumping the output into a file that can be previewed in VS Code for example.
    #[clap(long, conflicts_with = "url_encode")]
    pub markdown: bool,

    /// Encode the data to be included in a URL as expected by <https://mermaid.live/view>
    #[clap(long, conflicts_with = "markdown")]
    pub url_encode: bool,
}

pub(super) fn run(args: VisualizeCommand, ssa: Ssa) -> eyre::Result<()> {
    let mut output = render_mermaid(ssa).wrap_err("failed to render SSA to Mermaid")?;

    if args.markdown {
        output = format!("```mermaid\n{output}\n```");
    } else if args.url_encode {
        output = url_encode(output)?;
    }

    write_output(&output, args.output_path)
}

/// Render the SSA as a Mermaid [flowchart](https://mermaid.js.org/syntax/flowchart.html).
fn render_mermaid(ssa: Ssa) -> eyre::Result<String> {
    let indent = "    ";
    let mut out = String::new();
    let mut write_out = |i: usize, s: String| writeln!(out, "{}{s}", indent.repeat(i));

    write_out(0, "flowchart TB".into())?;

    // Defer rendering calls after all subgraphs have been defined,
    // otherwise Mermaid doesn't know that the target is going to
    // be a subgraph and treats it as a node.
    let mut calls = Vec::new();

    // Render each function as a subgraph, with internal jumps only.
    for (func_id, func) in ssa.functions {
        write_out(1, format!("subgraph {func_id} [{} {func_id}]", func.name()))?;
        let view = func.view();
        for block_id in view.blocks_iter() {
            let successors = view.block_successors_iter(block_id);

            // Show exit blocks as double circle, normal blocks as circles.
            let (bl, br) = if successors.len() == 0 { ("(((", ")))") } else { ("((", "))") };

            // Use the function ID to identify the block uniquely across all subgraphs.
            write_out(2, format!("{func_id}.{block_id}{bl}{block_id}{br}"))?;

            for successor_id in successors {
                write_out(2, format!("{func_id}.{block_id} --> {func_id}.{successor_id}"))?;
            }

            for callee_id in view.block_callees_iter(block_id) {
                calls.push((func_id, block_id, callee_id));
            }
        }
        write_out(1, "end\n".into())?;
    }

    // Render function calls.
    for (func_id, block_id, callee_id) in calls {
        write_out(1, format!("{func_id}.{block_id} -..-> {callee_id}"))?;
    }

    Ok(out)
}

/// Encode the Mermaid markup as it is expected by the [Mermaid Live Editor](https://mermaid.live)
///
/// The returned URL fragment should be used as `https://mermaid.live/view#{fragment}`
fn url_encode(markup: String) -> eyre::Result<String> {
    // See https://github.com/mermaid-js/mermaid-live-editor/discussions/1291
    let json = json!({
        "code": markup,
        "mermaid": {"theme": "default"}
    });
    let json = serde_json::to_string(&json)?;
    // We can use the `pako:` encoding for compression, or the `base64:` encoding for no-compression.
    let encoded = base64::engine::general_purpose::STANDARD.encode(json);
    let fragment = format!("base64:{encoded}");
    Ok(fragment)
}
