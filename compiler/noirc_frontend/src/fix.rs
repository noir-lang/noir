//! Rewrites source code to apply fixes for warnings whose fix is a pure removal: the code the
//! elaborator warned about is deleted or simplified, never added to.
//!
//! The entry point is [apply_fixes], which takes a file's source text, its parsed AST and the
//! set of [Fixes] to apply, and returns the rewritten source. The supported fixes are:
//!
//! - **Unused imports** (as reported by
//!   [UsageTracker::unused_imports][crate::usage_tracker::UsageTracker::unused_imports]).
//!   Composite `use` trees are rewritten rather than deleted: `use foo::{bar, baz};` with
//!   `baz` unused becomes `use foo::bar;`, and a `use` whose imports are all unused is
//!   deleted entirely (along with its line, when the line contains nothing else). Unused
//!   imports are identified by the imported name and its exact source location — the same
//!   `(Ident, Location)` key the usage tracker records — so two imports of the same name
//!   never alias each other.
//! - **Unnecessary `mut` modifiers**, identified by the location of the binding's
//!   identifier (the location `ResolverError::VariableDoesNotNeedToBeMutable` reports). The
//!   `mut` keyword is deleted and the binding is left untouched.
//!
//! Pruned `use` trees are re-rendered with [UseTree]'s `Display` impl, which produces a
//! single-line canonical form. Callers that want the result to respect a formatting style
//! should run the formatter afterwards.
//!
//! Fixing is deliberately a single round, not a fixpoint: it applies exactly the fixes the
//! elaborator reported for the given compilation. An import whose only consumer is another
//! `use` statement's path (resolving a path marks its first segment as used) is only
//! reported unused — and thus only removed — after the consuming `use` has been removed and
//! the program re-elaborated.

use std::collections::HashSet;
use std::ops::Range;

use noirc_errors::{Location, Span};

use crate::ast::{Ident, ItemVisibility, Path, Pattern, UseTree, UseTreeKind, Visitor};
use crate::parser::ParsedModule;

/// The warnings to fix, identified the same way the elaborator reports them.
#[derive(Debug, Default)]
pub struct Fixes {
    /// Unused imports, keyed by the imported name and its location — the same key
    /// [UsageTracker::unused_imports][crate::usage_tracker::UsageTracker::unused_imports]
    /// records.
    pub unused_imports: HashSet<(Ident, Location)>,
    /// Locations of binding identifiers whose `mut` modifier is unnecessary.
    pub unnecessary_muts: HashSet<Location>,
}

impl Fixes {
    pub fn is_empty(&self) -> bool {
        self.unused_imports.is_empty() && self.unnecessary_muts.is_empty()
    }
}

/// Returns `source` with all of `fixes` applied, or `None` if there was nothing to fix.
/// `parsed_module` must be the result of parsing `source`.
pub fn apply_fixes(source: &str, parsed_module: &ParsedModule, fixes: &Fixes) -> Option<String> {
    if fixes.is_empty() {
        return None;
    }

    let mut collector = FixCollector { fixes, replacements: Vec::new(), deletions: Vec::new() };
    parsed_module.accept(&mut collector);
    let FixCollector { replacements, mut deletions, .. } = collector;
    if replacements.is_empty() && deletions.is_empty() {
        return None;
    }

    // Deleted `use` items take their whole line with them. Runs of adjacent deleted lines are
    // merged so the blank-line handling in `swallow_extra_blank_line` sees the run as a unit.
    deletions = deletions
        .into_iter()
        .map(|range| extend_range_over_line(source, range))
        .collect::<Vec<_>>();
    deletions.sort_by_key(|range| range.start);
    let mut merged_deletions: Vec<Range<usize>> = Vec::new();
    for deletion in deletions {
        match merged_deletions.last_mut() {
            Some(last) if deletion.start <= last.end => last.end = last.end.max(deletion.end),
            _ => merged_deletions.push(deletion),
        }
    }

    let mut edits: Vec<(Range<usize>, String)> = merged_deletions
        .into_iter()
        .map(|range| (swallow_extra_blank_line(source, range), String::new()))
        .chain(replacements)
        .collect();

    // Apply the edits back to front so earlier edits' byte offsets stay valid.
    edits.sort_by_key(|(range, _)| range.start);
    let mut new_source = source.to_string();
    for (range, replacement) in edits.into_iter().rev() {
        new_source.replace_range(range, &replacement);
    }
    Some(new_source)
}

/// Walks the AST recording one edit per fixable warning: a replacement for a `use` item that
/// keeps some of its imports, a deletion of the whole item's span for a `use` item with no
/// imports left.
struct FixCollector<'a> {
    fixes: &'a Fixes,
    replacements: Vec<(Range<usize>, String)>,
    deletions: Vec<Range<usize>>,
}

impl Visitor for FixCollector<'_> {
    fn visit_import(&mut self, use_tree: &UseTree, span: Span, visibility: ItemVisibility) -> bool {
        let is_unused =
            |ident: &Ident| self.fixes.unused_imports.contains(&(ident.clone(), ident.location()));

        let (new_use_tree, removed_count) = use_tree_without_unused_imports(use_tree, &is_unused);
        if removed_count == 0 {
            return false;
        }

        let range = span.start() as usize..span.end() as usize;
        match new_use_tree {
            Some(use_tree) => {
                let replacement = if visibility == ItemVisibility::Private {
                    format!("use {use_tree};")
                } else {
                    format!("{visibility} use {use_tree};")
                };
                self.replacements.push((range, replacement));
            }
            None => self.deletions.push(range),
        }

        false
    }

    fn visit_mutable_pattern(
        &mut self,
        pattern: &Pattern,
        span: Span,
        is_synthesized: bool,
    ) -> bool {
        // `span` covers the whole `mut x` pattern while the identifier starts after the
        // `mut`, so deleting up to the identifier deletes exactly the modifier. Synthesized
        // patterns (e.g. desugared `self`) have no `mut` token in the source to delete.
        if !is_synthesized
            && let Pattern::Identifier(ident) = pattern
            && self.fixes.unnecessary_muts.contains(&ident.location())
        {
            let range = span.start() as usize..ident.location().span.start() as usize;
            self.replacements.push((range, String::new()));
        }

        true
    }
}

/// Returns a copy of `use_tree` with all unused imports removed, along with the number of
/// removed imports. Returns `None` for the tree if every import in it is unused.
///
/// `is_unused` is called with the [Ident] that the import binds: the alias if there is one,
/// the last path segment otherwise, and the segment before `self` for `self` imports — the
/// same ident whose location the usage tracker records.
pub fn use_tree_without_unused_imports(
    use_tree: &UseTree,
    is_unused: &dyn Fn(&Ident) -> bool,
) -> (Option<UseTree>, usize) {
    use_tree_without_unused_imports_impl(use_tree, None, is_unused)
}

fn use_tree_without_unused_imports_impl(
    use_tree: &UseTree,
    parent_prefix_last_ident: Option<&Ident>,
    is_unused: &dyn Fn(&Ident) -> bool,
) -> (Option<UseTree>, usize) {
    // The name a `self` import binds is the last segment of the accumulated prefix: for
    // `use foo::{self, ...}` the leaf's own prefix is empty and the name is the outer `foo`.
    let prefix_last_ident =
        use_tree.prefix.segments.last().map(|segment| &segment.ident).or(parent_prefix_last_ident);

    match &use_tree.kind {
        UseTreeKind::Path(name, alias) => {
            let binding = alias.as_ref().or(if name.as_str() == "self" {
                prefix_last_ident
            } else {
                Some(name)
            });
            if binding.is_some_and(is_unused) { (None, 1) } else { (Some(use_tree.clone()), 0) }
        }
        UseTreeKind::List(use_trees) => {
            let mut new_use_trees: Vec<UseTree> = Vec::new();
            let mut removed_count = 0;

            for use_tree in use_trees {
                let (new_use_tree, count) =
                    use_tree_without_unused_imports_impl(use_tree, prefix_last_ident, is_unused);
                if let Some(new_use_tree) = new_use_tree {
                    new_use_trees.push(new_use_tree);
                }
                removed_count += count;
            }

            let new_use_tree = if new_use_trees.is_empty() {
                None
            } else if new_use_trees.len() == 1 {
                Some(merge_prefix_into_tree(
                    &use_tree.prefix,
                    new_use_trees.remove(0),
                    use_tree.location,
                ))
            } else {
                Some(UseTree {
                    prefix: use_tree.prefix.clone(),
                    kind: UseTreeKind::List(new_use_trees),
                    location: use_tree.location,
                })
            };

            (new_use_tree, removed_count)
        }
    }
}

/// Unwraps the braces around a list that was left with a single entry, turning
/// `foo::{bar::baz}` into `foo::bar::baz`.
fn merge_prefix_into_tree(prefix: &Path, use_tree: UseTree, location: Location) -> UseTree {
    let mut prefix = prefix.clone();
    prefix.segments.extend(use_tree.prefix.segments);

    // `self` is only valid inside a bracketed list, so `foo::{self}` must become `foo`
    // (and `foo::{self as f}` must become `foo as f`), not `foo::self`.
    if let UseTreeKind::Path(name, alias) = &use_tree.kind
        && name.as_str() == "self"
        && let Some(last_segment) = prefix.segments.pop()
    {
        return UseTree {
            prefix,
            kind: UseTreeKind::Path(last_segment.ident, alias.clone()),
            location,
        };
    }

    UseTree { prefix, kind: use_tree.kind, location }
}

/// Given the byte range of a `use` item that is going to be deleted, extends the range over
/// the item's whole line (indentation and line terminator included) so no blank line is left
/// behind. Returns the range unchanged if the line contains anything else.
fn extend_range_over_line(source: &str, range: Range<usize>) -> Range<usize> {
    let bytes = source.as_bytes();

    // Walk backwards over the line's indentation; bail out if the item doesn't start the line.
    let mut start = range.start;
    while start > 0 && matches!(bytes[start - 1], b' ' | b'\t') {
        start -= 1;
    }
    if start != 0 && bytes[start - 1] != b'\n' {
        return range;
    }

    // Walk forwards over trailing whitespace and the line terminator; bail out if the line
    // has other content after the item.
    let mut end = range.end;
    while end < bytes.len() && matches!(bytes[end], b' ' | b'\t') {
        end += 1;
    }
    if end < bytes.len() && bytes[end] == b'\r' {
        end += 1;
    }
    if end < bytes.len() {
        if bytes[end] != b'\n' {
            return range;
        }
        end += 1;
    }

    start..end
}

/// Deleting a run of whole lines that sat between two blank lines (or between the start of
/// the file and a blank line) would leave a doubled-up blank line behind. When that is the
/// case, extends the range over the following blank line so a single blank line remains.
fn swallow_extra_blank_line(source: &str, range: Range<usize>) -> Range<usize> {
    let Range { start, mut end } = range;
    let bytes = source.as_bytes();

    let previous_line_is_blank =
        start == 0 || start == 1 || (start >= 2 && bytes[start - 2] == b'\n');
    if previous_line_is_blank {
        let mut blank_end = end;
        while blank_end < bytes.len() && matches!(bytes[blank_end], b' ' | b'\t') {
            blank_end += 1;
        }
        if blank_end < bytes.len() && bytes[blank_end] == b'\r' {
            blank_end += 1;
        }
        if blank_end < bytes.len() && bytes[blank_end] == b'\n' {
            end = blank_end + 1;
        }
    }

    start..end
}
