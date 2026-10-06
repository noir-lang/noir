//! Every combination of a Brillig array output's shape, the items of it that are constrained,
//! the way a value is read from it, and what that value is used for, checked as one table in
//! [`array_output_cases`].
//!
//! The `should report` column comes from which items a case constrains, not from the check
//! itself: a call should be reported if any value it returns may be unconstrained.
//! Where the check is known to be more lenient or stricter than that, the difference is
//! listed in [`known_differences`] and its reason is shown in the `why` column.
//!
//! To add a case, add a value to one of [`SHAPES`], [`CONSTRAINED`], [`READS`] or [`USES`]
//! and review the new rows of the table.
use std::fmt::Write;

use crate::ssa::{
    Ssa,
    ir::{instruction::Instruction, value::Value},
};

use super::{Context, DEFAULT_MAX_ANCESTOR_DISTANCE, DEFAULT_MAX_ARRAY_OUTPUT_LENGTH};

/// The type of the array returned by the `copy` call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shape {
    /// `[Field; N]`
    Flat(u32),
    /// `[(Field, Field); 2]`
    Tuples,
    /// `[[Field; 2]; 1]`
    Nested,
}

impl Shape {
    fn typ(self) -> String {
        match self {
            Shape::Flat(n) => format!("[Field; {n}]"),
            Shape::Tuples => "[(Field, Field); 2]".to_string(),
            Shape::Nested => "[[Field; 2]; 1]".to_string(),
        }
    }

    /// The number of `Field` values in the array.
    fn leaves(self) -> u32 {
        match self {
            Shape::Flat(n) => n,
            Shape::Tuples => 4,
            Shape::Nested => 2,
        }
    }

    /// Whether the array is longer than the check tracks item by item.
    fn is_large(self) -> bool {
        matches!(self, Shape::Flat(n) if n > DEFAULT_MAX_ARRAY_OUTPUT_LENGTH)
    }

    /// The leaves which the dynamic read in [`Read::Dynamic`] may return.
    fn dynamically_readable(self) -> Vec<u32> {
        match self {
            Shape::Flat(n) => (0..n).collect(),
            // The first field of either item.
            Shape::Tuples => vec![0, 2],
            // Either value of the only item.
            Shape::Nested => vec![0, 1],
        }
    }
}

/// Which leaves of the `copy` output are constrained against the input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Constrained {
    None,
    First,
    AllButLast,
    All,
}

impl Constrained {
    fn leaves(self, shape: Shape) -> Vec<u32> {
        let n = shape.leaves();
        match self {
            Constrained::None => vec![],
            Constrained::First => vec![0],
            Constrained::AllButLast => (0..n - 1).collect(),
            Constrained::All => (0..n).collect(),
        }
    }
}

/// How the value under test is read from the `copy` output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Read {
    /// The last leaf, at a constant index.
    LastLeaf,
    /// At an index only known at runtime.
    Dynamic,
    /// At an index only known at runtime, after the output of a `hint` call was written
    /// into the array at a constant index.
    DynamicAfterSet,
}

/// What the value read is used for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Use {
    /// Returned.
    Return,
    /// Passed into a `half` call, whose output is constrained against it.
    Half,
}

#[derive(Debug, Clone, Copy)]
struct Case {
    shape: Shape,
    constrained: Constrained,
    read: Read,
    used: Use,
}

impl Case {
    /// The calls which return values that may be unconstrained, by function name.
    fn expected(&self) -> Vec<&'static str> {
        let constrained = self.constrained.leaves(self.shape);
        let mut calls = Vec::new();
        if constrained.len() < self.shape.leaves() as usize {
            calls.push("copy");
        }
        if self.read == Read::DynamicAfterSet {
            calls.push("hint");
        }
        if self.used == Use::Half {
            let readable = match self.read {
                Read::LastLeaf => vec![self.shape.leaves() - 1],
                Read::Dynamic | Read::DynamicAfterSet => self.shape.dynamically_readable(),
            };
            let value_may_be_unconstrained = self.read == Read::DynamicAfterSet
                || readable.iter().any(|leaf| !constrained.contains(leaf));
            if value_may_be_unconstrained {
                calls.push("half");
            }
        }
        calls.sort_unstable();
        calls
    }

    fn ssa(&self) -> String {
        let mut ssa = SsaBuilder::default();
        let typ = self.shape.typ();
        let copy = ssa.call("f1", "v0", &typ);

        for leaf in self.constrained.leaves(self.shape) {
            let output = ssa.read_leaf(self.shape, &copy, leaf);
            let input = ssa.read_leaf(self.shape, "v0", leaf);
            writeln!(ssa.body, "    constrain {output} == {input}").unwrap();
        }

        let value = match self.read {
            Read::LastLeaf => ssa.read_leaf(self.shape, &copy, self.shape.leaves() - 1),
            Read::Dynamic => ssa.read_dynamic(self.shape, &copy),
            Read::DynamicAfterSet => {
                let hint = ssa.call("f3", "v2", "Field");
                let set = ssa.fresh();
                writeln!(ssa.body, "    {set} = array_set {copy}, index u32 0, value {hint}")
                    .unwrap();
                ssa.read_dynamic(self.shape, &set)
            }
        };

        let returned = match self.used {
            Use::Return => value,
            Use::Half => {
                let half = ssa.call("f2", &value, "Field");
                let double = ssa.fresh();
                writeln!(ssa.body, "    {double} = mul {half}, Field 2").unwrap();
                writeln!(ssa.body, "    constrain {double} == {value}").unwrap();
                half
            }
        };

        format!(
            "acir(inline) fn main f0 {{
  b0(v0: {typ}, v1: u32, v2: Field):
{body}    return {returned}
}}

brillig(inline) fn copy f1 {{
  b0(v0: {typ}):
    return v0
}}

brillig(inline) fn half f2 {{
  b0(v0: Field):
    v2 = div v0, Field 2
    return v2
}}

brillig(inline) fn hint f3 {{
  b0(v0: Field):
    return v0
}}
",
            body = ssa.body
        )
    }
}

/// Appends instructions to the body of `main`, whose parameters are `v0`, `v1` and `v2`.
struct SsaBuilder {
    body: String,
    next: u32,
}

impl Default for SsaBuilder {
    fn default() -> Self {
        Self { body: String::new(), next: 3 }
    }
}

impl SsaBuilder {
    fn fresh(&mut self) -> String {
        let value = format!("v{}", self.next);
        self.next += 1;
        value
    }

    fn call(&mut self, function: &str, argument: &str, typ: &str) -> String {
        let result = self.fresh();
        writeln!(self.body, "    {result} = call {function}({argument}) -> {typ}").unwrap();
        result
    }

    fn array_get(&mut self, array: &str, index: &str, typ: &str) -> String {
        let result = self.fresh();
        writeln!(self.body, "    {result} = array_get {array}, index {index} -> {typ}").unwrap();
        result
    }

    /// Read a leaf at a constant index.
    fn read_leaf(&mut self, shape: Shape, array: &str, leaf: u32) -> String {
        match shape {
            // Tuples are flattened into the array.
            Shape::Flat(_) | Shape::Tuples => {
                self.array_get(array, &format!("u32 {leaf}"), "Field")
            }
            Shape::Nested => {
                let item = self.array_get(array, &format!("u32 {}", leaf / 2), "[Field; 2]");
                self.array_get(&item, &format!("u32 {}", leaf % 2), "Field")
            }
        }
    }

    /// Read one of [`Shape::dynamically_readable`] at the index `v1`.
    fn read_dynamic(&mut self, shape: Shape, array: &str) -> String {
        match shape {
            Shape::Flat(_) => self.array_get(array, "v1", "Field"),
            Shape::Tuples => {
                let index = self.fresh();
                writeln!(self.body, "    {index} = mul v1, u32 2").unwrap();
                self.array_get(array, &index, "Field")
            }
            Shape::Nested => {
                let item = self.array_get(array, "u32 0", "[Field; 2]");
                self.array_get(&item, "v1", "Field")
            }
        }
    }
}

/// Where the check is known to differ from [`Case::expected`]: the calls it does not report
/// although they should be, and the calls it reports although they need not be.
struct Difference {
    missed: &'static [&'static str],
    extra: &'static [&'static str],
    reason: &'static str,
}

fn known_differences(case: &Case) -> Option<Difference> {
    let Case { shape, constrained, read, used } = *case;
    let partial = matches!(constrained, Constrained::First | Constrained::AllButLast);

    // A large array is tracked as a single value, so it is cleared by a constraint on any one
    // of its items, and so is everything read from it at a constant index.
    if shape.is_large() && partial {
        let missed: &[&str] =
            if read == Read::LastLeaf && used == Use::Half { &["copy", "half"] } else { &["copy"] };
        return Some(Difference {
            missed,
            extra: &[],
            reason: "noir-claude#917: large array tracked as one value",
        });
    }

    // An item which is an array is tracked as a single value, so it is cleared by a constraint
    // on any one of its values, and so is everything read from it at a constant index.
    if shape == Shape::Nested && partial {
        let missed: &[&str] =
            if read == Read::LastLeaf && used == Use::Half { &["copy", "half"] } else { &["copy"] };
        return Some(Difference {
            missed,
            extra: &[],
            reason: "noir-claude#918: inner array tracked as one value",
        });
    }

    // A value read at a dynamic index only counts as constrained when every item of an array
    // output of numeric items is: a large array, or an item which is itself an array, is
    // cleared as soon as any one of its values is constrained, so it is never relied on.
    if (shape.is_large() || shape == Shape::Nested)
        && constrained == Constrained::All
        && read == Read::Dynamic
        && used == Use::Half
    {
        return Some(Difference {
            missed: &[],
            extra: &["half"],
            reason: "never known to be fully constrained",
        });
    }

    // A value read at a dynamic index only counts as constrained when every value in the
    // array is, even if the index can only select values which are constrained.
    if shape == Shape::Tuples
        && constrained == Constrained::AllButLast
        && read == Read::Dynamic
        && used == Use::Half
    {
        return Some(Difference {
            missed: &[],
            extra: &["half"],
            reason: "dynamic read needs the whole array constrained",
        });
    }

    None
}

/// The names of the functions called by the calls the check reports.
fn reported(src: &str) -> Vec<String> {
    let ssa = Ssa::from_str(src).unwrap();
    let main = ssa.main();
    let context =
        Context::new(main, DEFAULT_MAX_ARRAY_OUTPUT_LENGTH, DEFAULT_MAX_ANCESTOR_DISTANCE)
            .build_tainted(main, &ssa.functions)
            .build_parent_graph(main)
            .constrain_tainted(main, &ssa.functions);
    let mut names: Vec<String> = context
        .tainted
        .unresolved_instructions()
        .map(|instruction| {
            let Instruction::Call { func, .. } = &main.dfg[instruction] else {
                panic!("expected a call");
            };
            let Value::Function(id) = &main.dfg[*func] else {
                panic!("expected a function");
            };
            ssa.functions[id].name().to_string()
        })
        .collect();
    names.sort_unstable();
    names
}

fn apply(expected: Vec<&'static str>, difference: &Difference) -> Vec<&'static str> {
    let mut calls: Vec<&str> =
        expected.into_iter().filter(|call| !difference.missed.contains(call)).collect();
    calls.extend(difference.extra);
    calls.sort_unstable();
    calls
}

/// An array output longer than the check tracks item by item.
const LARGE: u32 = DEFAULT_MAX_ARRAY_OUTPUT_LENGTH + 1;

const SHAPES: [Shape; 4] = [Shape::Flat(2), Shape::Flat(LARGE), Shape::Tuples, Shape::Nested];
const CONSTRAINED: [Constrained; 4] =
    [Constrained::None, Constrained::First, Constrained::AllButLast, Constrained::All];
const READS: [Read; 3] = [Read::LastLeaf, Read::Dynamic, Read::DynamicAfterSet];
const USES: [Use; 2] = [Use::Return, Use::Half];

fn cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for shape in SHAPES {
        for constrained in CONSTRAINED {
            for read in READS {
                // The value written at index 0 is a `Field`, which is only an item of flat arrays.
                if read == Read::DynamicAfterSet && !matches!(shape, Shape::Flat(_)) {
                    continue;
                }
                for used in USES {
                    cases.push(Case { shape, constrained, read, used });
                }
            }
        }
    }
    cases
}

/// A leaf of `c`, the output of `copy`, in Noir syntax.
fn leaf_name(shape: Shape, leaf: u32) -> String {
    match shape {
        Shape::Flat(_) => format!("c[{leaf}]"),
        Shape::Tuples => format!("c[{}].{}", leaf / 2, leaf % 2),
        Shape::Nested => format!("c[{}][{}]", leaf / 2, leaf % 2),
    }
}

impl Case {
    /// The leaves constrained against the input, in Noir syntax.
    fn constrained_column(&self) -> String {
        let last = leaf_name(self.shape, self.shape.leaves() - 1);
        match self.constrained {
            Constrained::None => "none".to_string(),
            Constrained::First => leaf_name(self.shape, 0),
            Constrained::AllButLast => format!("all but {last}"),
            Constrained::All => "all".to_string(),
        }
    }

    /// The value read from `c`, in Noir syntax, with `i` only known at runtime.
    fn value_column(&self) -> String {
        let dynamic = match self.shape {
            Shape::Flat(_) => "c[i]",
            Shape::Tuples => "c[i].0",
            Shape::Nested => "c[0][i]",
        };
        match self.read {
            Read::LastLeaf => leaf_name(self.shape, self.shape.leaves() - 1),
            Read::Dynamic => dynamic.to_string(),
            Read::DynamicAfterSet => format!("{dynamic} after c[0] = hint(y)"),
        }
    }

    fn use_column(&self) -> &'static str {
        match self.used {
            Use::Return => "return",
            Use::Half => "half(value)",
        }
    }
}

fn calls_column<S: AsRef<str>>(calls: &[S]) -> String {
    if calls.is_empty() {
        "-".to_string()
    } else {
        calls.iter().map(AsRef::as_ref).collect::<Vec<_>>().join(" ")
    }
}

/// One row per case: the `copy` output's type, which of its leaves are constrained, the value
/// read from it and its use, then the calls which should be reported because they return a
/// value that may be unconstrained, the calls the check reports, and why they differ.
///
/// A row whose `should report` and `reports` columns differ must have a reason from
/// [`known_differences`], and a reason must only be given where they differ.
#[test]
fn array_output_cases() {
    let header = ["copy returns", "constrained", "value", "use", "should report", "reports", "why"];
    let mut rows = vec![header.map(String::from).to_vec()];
    let mut unexplained = Vec::new();
    for case in cases() {
        let src = case.ssa();
        let expected = case.expected();
        let reported = reported(&src);
        let difference = known_differences(&case);
        let reason = difference.as_ref().map_or("", |difference| difference.reason);
        let explained = match &difference {
            Some(difference) => apply(expected.clone(), difference),
            None => expected.clone(),
        };
        if reported != explained || (difference.is_some() && explained == expected) {
            unexplained.push(format!("{case:?}\n{src}"));
        }
        rows.push(vec![
            case.shape.typ(),
            case.constrained_column(),
            case.value_column(),
            case.use_column().to_string(),
            calls_column(&expected),
            calls_column(&reported),
            reason.to_string(),
        ]);
    }

    let widths: Vec<usize> = (0..header.len())
        .map(|column| rows.iter().map(|row| row[column].chars().count()).max().unwrap())
        .collect();
    let mut table = String::new();
    for row in rows {
        let cells: Vec<String> =
            row.iter().zip(&widths).map(|(cell, width)| format!("{cell:<width$}")).collect();
        writeln!(table, "{}", cells.join(" | ").trim_end()).unwrap();
    }
    insta::assert_snapshot!(table, @r"
    copy returns        | constrained     | value                     | use         | should report  | reports        | why
    [Field; 2]          | none            | c[1]                      | return      | copy           | copy           |
    [Field; 2]          | none            | c[1]                      | half(value) | copy half      | copy half      |
    [Field; 2]          | none            | c[i]                      | return      | copy           | copy           |
    [Field; 2]          | none            | c[i]                      | half(value) | copy half      | copy half      |
    [Field; 2]          | none            | c[i] after c[0] = hint(y) | return      | copy hint      | copy hint      |
    [Field; 2]          | none            | c[i] after c[0] = hint(y) | half(value) | copy half hint | copy half hint |
    [Field; 2]          | c[0]            | c[1]                      | return      | copy           | copy           |
    [Field; 2]          | c[0]            | c[1]                      | half(value) | copy half      | copy half      |
    [Field; 2]          | c[0]            | c[i]                      | return      | copy           | copy           |
    [Field; 2]          | c[0]            | c[i]                      | half(value) | copy half      | copy half      |
    [Field; 2]          | c[0]            | c[i] after c[0] = hint(y) | return      | copy hint      | copy hint      |
    [Field; 2]          | c[0]            | c[i] after c[0] = hint(y) | half(value) | copy half hint | copy half hint |
    [Field; 2]          | all but c[1]    | c[1]                      | return      | copy           | copy           |
    [Field; 2]          | all but c[1]    | c[1]                      | half(value) | copy half      | copy half      |
    [Field; 2]          | all but c[1]    | c[i]                      | return      | copy           | copy           |
    [Field; 2]          | all but c[1]    | c[i]                      | half(value) | copy half      | copy half      |
    [Field; 2]          | all but c[1]    | c[i] after c[0] = hint(y) | return      | copy hint      | copy hint      |
    [Field; 2]          | all but c[1]    | c[i] after c[0] = hint(y) | half(value) | copy half hint | copy half hint |
    [Field; 2]          | all             | c[1]                      | return      | -              | -              |
    [Field; 2]          | all             | c[1]                      | half(value) | -              | -              |
    [Field; 2]          | all             | c[i]                      | return      | -              | -              |
    [Field; 2]          | all             | c[i]                      | half(value) | -              | -              |
    [Field; 2]          | all             | c[i] after c[0] = hint(y) | return      | hint           | hint           |
    [Field; 2]          | all             | c[i] after c[0] = hint(y) | half(value) | half hint      | half hint      |
    [Field; 65]         | none            | c[64]                     | return      | copy           | copy           |
    [Field; 65]         | none            | c[64]                     | half(value) | copy half      | copy half      |
    [Field; 65]         | none            | c[i]                      | return      | copy           | copy           |
    [Field; 65]         | none            | c[i]                      | half(value) | copy half      | copy half      |
    [Field; 65]         | none            | c[i] after c[0] = hint(y) | return      | copy hint      | copy hint      |
    [Field; 65]         | none            | c[i] after c[0] = hint(y) | half(value) | copy half hint | copy half hint |
    [Field; 65]         | c[0]            | c[64]                     | return      | copy           | -              | noir-claude#917: large array tracked as one value
    [Field; 65]         | c[0]            | c[64]                     | half(value) | copy half      | -              | noir-claude#917: large array tracked as one value
    [Field; 65]         | c[0]            | c[i]                      | return      | copy           | -              | noir-claude#917: large array tracked as one value
    [Field; 65]         | c[0]            | c[i]                      | half(value) | copy half      | half           | noir-claude#917: large array tracked as one value
    [Field; 65]         | c[0]            | c[i] after c[0] = hint(y) | return      | copy hint      | hint           | noir-claude#917: large array tracked as one value
    [Field; 65]         | c[0]            | c[i] after c[0] = hint(y) | half(value) | copy half hint | half hint      | noir-claude#917: large array tracked as one value
    [Field; 65]         | all but c[64]   | c[64]                     | return      | copy           | -              | noir-claude#917: large array tracked as one value
    [Field; 65]         | all but c[64]   | c[64]                     | half(value) | copy half      | -              | noir-claude#917: large array tracked as one value
    [Field; 65]         | all but c[64]   | c[i]                      | return      | copy           | -              | noir-claude#917: large array tracked as one value
    [Field; 65]         | all but c[64]   | c[i]                      | half(value) | copy half      | half           | noir-claude#917: large array tracked as one value
    [Field; 65]         | all but c[64]   | c[i] after c[0] = hint(y) | return      | copy hint      | hint           | noir-claude#917: large array tracked as one value
    [Field; 65]         | all but c[64]   | c[i] after c[0] = hint(y) | half(value) | copy half hint | half hint      | noir-claude#917: large array tracked as one value
    [Field; 65]         | all             | c[64]                     | return      | -              | -              |
    [Field; 65]         | all             | c[64]                     | half(value) | -              | -              |
    [Field; 65]         | all             | c[i]                      | return      | -              | -              |
    [Field; 65]         | all             | c[i]                      | half(value) | -              | half           | never known to be fully constrained
    [Field; 65]         | all             | c[i] after c[0] = hint(y) | return      | hint           | hint           |
    [Field; 65]         | all             | c[i] after c[0] = hint(y) | half(value) | half hint      | half hint      |
    [(Field, Field); 2] | none            | c[1].1                    | return      | copy           | copy           |
    [(Field, Field); 2] | none            | c[1].1                    | half(value) | copy half      | copy half      |
    [(Field, Field); 2] | none            | c[i].0                    | return      | copy           | copy           |
    [(Field, Field); 2] | none            | c[i].0                    | half(value) | copy half      | copy half      |
    [(Field, Field); 2] | c[0].0          | c[1].1                    | return      | copy           | copy           |
    [(Field, Field); 2] | c[0].0          | c[1].1                    | half(value) | copy half      | copy half      |
    [(Field, Field); 2] | c[0].0          | c[i].0                    | return      | copy           | copy           |
    [(Field, Field); 2] | c[0].0          | c[i].0                    | half(value) | copy half      | copy half      |
    [(Field, Field); 2] | all but c[1].1  | c[1].1                    | return      | copy           | copy           |
    [(Field, Field); 2] | all but c[1].1  | c[1].1                    | half(value) | copy half      | copy half      |
    [(Field, Field); 2] | all but c[1].1  | c[i].0                    | return      | copy           | copy           |
    [(Field, Field); 2] | all but c[1].1  | c[i].0                    | half(value) | copy           | copy half      | dynamic read needs the whole array constrained
    [(Field, Field); 2] | all             | c[1].1                    | return      | -              | -              |
    [(Field, Field); 2] | all             | c[1].1                    | half(value) | -              | -              |
    [(Field, Field); 2] | all             | c[i].0                    | return      | -              | -              |
    [(Field, Field); 2] | all             | c[i].0                    | half(value) | -              | -              |
    [[Field; 2]; 1]     | none            | c[0][1]                   | return      | copy           | copy           |
    [[Field; 2]; 1]     | none            | c[0][1]                   | half(value) | copy half      | copy half      |
    [[Field; 2]; 1]     | none            | c[0][i]                   | return      | copy           | copy           |
    [[Field; 2]; 1]     | none            | c[0][i]                   | half(value) | copy half      | copy half      |
    [[Field; 2]; 1]     | c[0][0]         | c[0][1]                   | return      | copy           | -              | noir-claude#918: inner array tracked as one value
    [[Field; 2]; 1]     | c[0][0]         | c[0][1]                   | half(value) | copy half      | -              | noir-claude#918: inner array tracked as one value
    [[Field; 2]; 1]     | c[0][0]         | c[0][i]                   | return      | copy           | -              | noir-claude#918: inner array tracked as one value
    [[Field; 2]; 1]     | c[0][0]         | c[0][i]                   | half(value) | copy half      | half           | noir-claude#918: inner array tracked as one value
    [[Field; 2]; 1]     | all but c[0][1] | c[0][1]                   | return      | copy           | -              | noir-claude#918: inner array tracked as one value
    [[Field; 2]; 1]     | all but c[0][1] | c[0][1]                   | half(value) | copy half      | -              | noir-claude#918: inner array tracked as one value
    [[Field; 2]; 1]     | all but c[0][1] | c[0][i]                   | return      | copy           | -              | noir-claude#918: inner array tracked as one value
    [[Field; 2]; 1]     | all but c[0][1] | c[0][i]                   | half(value) | copy half      | half           | noir-claude#918: inner array tracked as one value
    [[Field; 2]; 1]     | all             | c[0][1]                   | return      | -              | -              |
    [[Field; 2]; 1]     | all             | c[0][1]                   | half(value) | -              | -              |
    [[Field; 2]; 1]     | all             | c[0][i]                   | return      | -              | -              |
    [[Field; 2]; 1]     | all             | c[0][i]                   | half(value) | -              | half           | never known to be fully constrained
    ");

    assert!(
        unexplained.is_empty(),
        "the reported calls differ from the expected ones without a matching known difference:\n{}",
        unexplained.join("\n")
    );
}
