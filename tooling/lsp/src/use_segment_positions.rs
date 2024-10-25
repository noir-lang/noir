use std::collections::HashMap;

use noirc_errors::Span;
use noirc_frontend::ast::{PathKind, UseTree, UseTreeKind};

/// The position of a segment in a `use` statement.
/// We use this to determine how an auto-import should be inserted.
#[derive(Debug, Default, Copy, Clone)]
pub(crate) enum UseSegmentPosition {
    /// The segment either doesn't exist in the source code or there are multiple segments.
    /// In this case auto-import will add a new use statement.
    #[default]
    NoneOrMultiple,
    /// The segment is the last one in the `use` statement (or nested use statement):
    ///
    /// use foo::bar;
    ///          ^^^
    ///
    /// Auto-import will transform it to this:
    ///
    /// use foo::bar::{self, baz};
    Last { span: Span },
    /// The segment happens before another simple (ident) segment:
    ///
    /// use foo::bar::qux;
    ///          ^^^
    ///
    /// Auto-import will transform it to this:
    ///
    /// use foo::bar::{qux, baz};
    BeforeSegment { segment_span_until_end: Span },
    /// The segment happens before a list:
    ///
    /// use foo::bar::{qux, another};
    ///
    /// Auto-import will transform it to this:
    ///
    /// use foo::bar::{qux, another, baz};
    BeforeList { first_entry_span: Span, list_is_empty: bool },
}

/// Remembers where each segment in a `use` statement is located.
/// The key is the full segment, so for `use foo::bar::baz` we'll have three
/// segments: `foo`, `foo::bar` and `foo::bar::baz`, where the span is just
/// for the last identifier (`foo`, `bar` and `baz` in the previous example).
#[derive(Default)]
pub(crate) struct UseSegmentPositions {
    use_segment_positions: HashMap<String, UseSegmentPosition>,
}

impl UseSegmentPositions {
    pub(crate) fn add(&mut self, use_tree: &UseTree) {
        self.gather_use_tree_segments(use_tree, String::new());
    }

    /// Given a full path like `foo::bar::baz`, returns the first non-"NoneOrMultiple" segment position
    /// trying each successive parent, together with the name after the parent.
    ///
    /// For example, first we'll check if `foo::bar` has a single position. If not, we'll try with `foo`.
    pub(crate) fn get(&self, full_path: &str) -> (UseSegmentPosition, String) {
        // Build a parent path to know in which full segment we need to add this import
        let mut segments: Vec<_> = full_path.split("::").collect();
        let mut name = segments.pop().unwrap().to_string();
        let mut parent_path = segments.join("::");

        loop {
            let use_segment_position =
                self.use_segment_positions.get(&parent_path).cloned().unwrap_or_default();

            if let UseSegmentPosition::NoneOrMultiple = use_segment_position {
                if let Some(next_name) = segments.pop() {
                    name = format!("{next_name}::{name}");
                    parent_path = segments.join("::");
                } else {
                    return (UseSegmentPosition::NoneOrMultiple, String::new());
                }
            } else {
                return (use_segment_position, name);
            }
        }
    }

    fn gather_use_tree_segments(&mut self, use_tree: &UseTree, mut prefix: String) {
        let kind_string = match use_tree.prefix.kind {
            PathKind::Crate => Some("crate".to_string()),
            PathKind::Super => Some("super".to_string()),
            PathKind::Dep | PathKind::Plain => None,
        };
        if let Some(kind_string) = kind_string {
            if let Some(segment) = use_tree.prefix.segments.first() {
                self.insert_use_segment_position(
                    kind_string,
                    UseSegmentPosition::BeforeSegment {
                        segment_span_until_end: Span::from(
                            segment.ident.span().start()..use_tree.span.end() - 1,
                        ),
                    },
                );
            } else {
                self.insert_use_segment_position_before_use_tree_kind(use_tree, kind_string);
            }
        }

        let prefix_segments_len = use_tree.prefix.segments.len();
        for (index, segment) in use_tree.prefix.segments.iter().enumerate() {
            let ident = &segment.ident;
            if !prefix.is_empty() {
                prefix.push_str("::");
            };
            prefix.push_str(&ident.0.contents);

            if index < prefix_segments_len - 1 {
                self.insert_use_segment_position(
                    prefix.clone(),
                    UseSegmentPosition::BeforeSegment {
                        segment_span_until_end: Span::from(
                            use_tree.prefix.segments[index + 1].ident.span().start()
                                ..use_tree.span.end() - 1,
                        ),
                    },
                );
            } else {
                self.insert_use_segment_position_before_use_tree_kind(use_tree, prefix.clone());
            }
        }

        match &use_tree.kind {
            UseTreeKind::Path(ident, alias) => {
                if !prefix.is_empty() {
                    prefix.push_str("::");
                }
                prefix.push_str(&ident.0.contents);

                if alias.is_none() {
                    self.insert_use_segment_position(
                        prefix,
                        UseSegmentPosition::Last { span: ident.span() },
                    );
                } else {
                    self.insert_use_segment_position(prefix, UseSegmentPosition::NoneOrMultiple);
                }
            }
            UseTreeKind::List(use_trees) => {
                for use_tree in use_trees {
                    self.gather_use_tree_segments(use_tree, prefix.clone());
                }
            }
        }
    }

    fn insert_use_segment_position_before_use_tree_kind(
        &mut self,
        use_tree: &UseTree,
        prefix: String,
    ) {
        match &use_tree.kind {
            UseTreeKind::Path(ident, _alias) => {
                self.insert_use_segment_position(
                    prefix,
                    UseSegmentPosition::BeforeSegment {
                        segment_span_until_end: Span::from(
                            ident.span().start()..use_tree.span.end() - 1,
                        ),
                    },
                );
            }
            UseTreeKind::List(use_trees) => {
                if let Some(first_use_tree) = use_trees.first() {
                    self.insert_use_segment_position(
                        prefix,
                        UseSegmentPosition::BeforeList {
                            first_entry_span: first_use_tree.prefix.span(),
                            list_is_empty: false,
                        },
                    );
                } else {
                    self.insert_use_segment_position(
                        prefix,
                        UseSegmentPosition::BeforeList {
                            first_entry_span: Span::from(
                                use_tree.span.end() - 1..use_tree.span.end() - 1,
                            ),
                            list_is_empty: true,
                        },
                    );
                }
            }
        }
    }

    fn insert_use_segment_position(&mut self, segment: String, position: UseSegmentPosition) {
        if self.use_segment_positions.get(&segment).is_none() {
            self.use_segment_positions.insert(segment, position);
        } else {
            self.use_segment_positions.insert(segment, UseSegmentPosition::NoneOrMultiple);
        }
    }
}
