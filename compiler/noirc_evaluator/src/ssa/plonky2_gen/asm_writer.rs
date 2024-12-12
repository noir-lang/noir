use std::{
    borrow::Borrow,
    collections::{HashMap, VecDeque},
    fmt::Display,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    str::FromStr,
};

use codespan_reporting::files::Files;
use fm::{FileId, FileMap, PathString};
use plonky2::iop::{
    target::{BoolTarget, Target},
    wire::Wire,
};
use plonky2_u32::gadgets::arithmetic_u32::{CircuitBuilderU32, U32Target};
use serde::{Deserialize, Serialize};
use tracing::Instrument;

use crate::{
    debug_trace::{AsmListIndexRange, DebugTraceList, SourcePoint},
    ssa::ir::dfg::CallStack,
};

use super::config::P2Builder;
use super::config::P2Field;

struct TargetDisplay {
    t: Target,
}

impl Display for TargetDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.t {
            Target::VirtualTarget { index } => {
                write!(f, "v{}", index)
            }
            Target::Wire(Wire { row, column }) => {
                write!(f, "r{}c{}", row, column)
            }
        }
    }
}
struct VecTargetDisplay<'a> {
    t: &'a Vec<Target>,
}

impl<'a> Display for VecTargetDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        let mut first = true;
        for tt in self.t {
            if first {
                write!(f, "{}", TargetDisplay { t: *tt })?;
                first = false;
            } else {
                write!(f, ",{}", TargetDisplay { t: *tt })?;
            }
        }
        write!(f, ")")?;
        Ok(())
    }
}

struct BoolTargetDisplay {
    t: BoolTarget,
}

impl Display for BoolTargetDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", TargetDisplay { t: self.t.target })
    }
}

struct VecBoolTargetDisplay<'a> {
    t: &'a Vec<BoolTarget>,
}

impl<'a> Display for VecBoolTargetDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        let mut first = true;
        for bt in self.t {
            if first {
                write!(f, "{}", BoolTargetDisplay { t: *bt })?;
                first = false;
            } else {
                write!(f, ",{}", BoolTargetDisplay { t: *bt })?;
            }
        }
        write!(f, ")")?;
        Ok(())
    }
}

struct TargetSliceDisplay<'a> {
    t: &'a [Target],
}

impl<'a> Display for TargetSliceDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        let mut first = true;
        for tt in self.t {
            if first {
                write!(f, "{}", TargetDisplay { t: *tt })?;
                first = false;
            } else {
                write!(f, ",{}", TargetDisplay { t: *tt })?;
            }
        }
        write!(f, ")")?;
        Ok(())
    }
}

struct TargetIntoIteratorDisplay<T, TIter>
where
    TIter: IntoIterator<Item = T> + Clone,
    T: Borrow<Target>,
{
    t: TIter,
}

impl<TIter: IntoIterator<Item = T> + Clone, T: Borrow<Target>> Display
    for TargetIntoIteratorDisplay<T, TIter>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        let mut first = true;
        for tt in self.t.clone() {
            if first {
                write!(f, "{}", TargetDisplay { t: *tt.borrow() })?;
                first = false;
            } else {
                write!(f, ",{}", TargetDisplay { t: *tt.borrow() })?;
            }
        }
        write!(f, ")")?;
        Ok(())
    }
}

struct BoolTargetIteratorDisplay<T, TIter>
where
    TIter: Iterator<Item = T> + Clone,
    T: Borrow<BoolTarget>,
{
    t: TIter,
}

impl<TIter: Iterator<Item = T> + Clone, T: Borrow<BoolTarget>> Display
    for BoolTargetIteratorDisplay<T, TIter>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        let mut first = true;
        for tt in self.t.clone() {
            if first {
                write!(f, "{}", BoolTargetDisplay { t: *tt.borrow() })?;
                first = false;
            } else {
                write!(f, ",{}", BoolTargetDisplay { t: *tt.borrow() })?;
            }
        }
        write!(f, ")")?;
        Ok(())
    }
}

struct U32TargetDisplay {
    t: U32Target,
}

impl Display for U32TargetDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", TargetDisplay { t: self.t.0 })
    }
}

pub struct AsmWriter {
    pub builder: P2Builder,
    pub show_plonky2: bool,
    file: Option<BufWriter<File>>,
    last_call_stack: CallStack,
    last_file: FileId,
    last_line_number_begin: usize,
    last_line_number_end: usize,
    file_map: FileMap,
    // information, stored, for use with the 'nargo trace --trace-plonky2' feature
    pub debug_trace_list: Option<DebugTraceList>,
    prev_source_point: Option<SourcePoint>,
}

impl AsmWriter {
    fn output_enabled(&self) -> bool {
        self.show_plonky2 || self.file.is_some() || self.debug_trace_list.is_some()
    }

    fn handle_output(&mut self, s: String) {
        if self.show_plonky2 {
            println!("{}", s);
        }

        if let Some(f) = &mut self.file {
            writeln!(f, "{}", s).expect("Unable to write PLONKY2 data to file");
        }

        if let Some(l) = &mut self.debug_trace_list {
            l.list.push(s);
        }
    }

    pub fn get_builder(&self) -> &P2Builder {
        &self.builder
    }
    pub fn get_mut_builder(&mut self) -> &mut P2Builder {
        &mut self.builder
    }
    pub fn move_builder(self) -> P2Builder {
        self.builder
    }

    pub fn new(
        builder: P2Builder,
        show_plonky2: bool,
        plonky2_print_file: Option<String>,
        file_map: FileMap,
        create_debug_trace_list: bool,
    ) -> Self {
        AsmWriter {
            builder,
            show_plonky2,
            file: if let Some(file_name) = plonky2_print_file {
                Some(BufWriter::new(
                    File::create(file_name).expect("Unable to create PLONKY2 file"),
                ))
            } else {
                None
            },
            last_call_stack: CallStack::default(),
            last_file: FileId::default(),
            last_line_number_begin: 0,
            last_line_number_end: 0,
            file_map,
            debug_trace_list: if create_debug_trace_list {
                Some(DebugTraceList::new())
            } else {
                None
            },
            prev_source_point: None,
        }
    }

    pub fn is_equal(&mut self, x: Target, y: Target) -> BoolTarget {
        let result = self.builder.is_equal(x, y);
        if self.output_enabled() {
            self.handle_output(format!(
                "{} = is_equal {},{}",
                BoolTargetDisplay { t: result },
                TargetDisplay { t: x },
                TargetDisplay { t: y },
            ));
        }
        result
    }

    pub fn zero(&mut self) -> Target {
        let result = self.builder.zero();
        if self.output_enabled() {
            self.handle_output(format!("{} = zero", TargetDisplay { t: result }));
        }
        result
    }

    pub fn one(&mut self) -> Target {
        let result = self.builder.one();
        if self.output_enabled() {
            self.handle_output(format!("{} = one", TargetDisplay { t: result }));
        }
        result
    }

    pub fn two(&mut self) -> Target {
        let result = self.builder.two();
        if self.output_enabled() {
            self.handle_output(format!("{} = two", TargetDisplay { t: result }));
        }
        result
    }

    pub fn split_le(&mut self, integer: Target, num_bits: usize) -> Vec<BoolTarget> {
        let result = self.builder.split_le(integer, num_bits);
        if self.output_enabled() {
            self.handle_output(format!(
                "{} = split_le {},{}",
                VecBoolTargetDisplay { t: &result },
                TargetDisplay { t: integer },
                num_bits,
            ));
        }
        result
    }

    pub fn _if(&mut self, b: BoolTarget, x: Target, y: Target) -> Target {
        let result = self.builder._if(b, x, y);
        if self.output_enabled() {
            self.handle_output(format!(
                "{} = if {},{},{}",
                TargetDisplay { t: result },
                BoolTargetDisplay { t: b },
                TargetDisplay { t: x },
                TargetDisplay { t: y },
            ));
        }
        result
    }

    pub fn exp_u64(&mut self, base: Target, exponent: u64) -> Target {
        let result = self.builder.exp_u64(base, exponent);
        if self.output_enabled() {
            self.handle_output(format!(
                "{} = exp_u64 {},{}",
                TargetDisplay { t: result },
                TargetDisplay { t: base },
                exponent,
            ));
        }
        result
    }

    pub fn constant(&mut self, c: P2Field) -> Target {
        let result = self.builder.constant(c);
        if self.output_enabled() {
            self.handle_output(format!("{} = constant {}", TargetDisplay { t: result }, c));
        }
        result
    }

    pub fn constant_bool(&mut self, b: bool) -> BoolTarget {
        let result = self.builder.constant_bool(b);
        if self.output_enabled() {
            self.handle_output(format!(
                "{} = constant_bool {}",
                BoolTargetDisplay { t: result },
                b
            ));
        }
        result
    }

    pub fn mul(&mut self, x: Target, y: Target) -> Target {
        let result = self.builder.mul(x, y);
        if self.output_enabled() {
            self.handle_output(format!(
                "{} = mul {},{}",
                TargetDisplay { t: result },
                TargetDisplay { t: x },
                TargetDisplay { t: y },
            ));
        }
        result
    }

    pub fn and(&mut self, b1: BoolTarget, b2: BoolTarget) -> BoolTarget {
        let result = self.builder.and(b1, b2);
        if self.output_enabled() {
            self.handle_output(format!(
                "{} = and {},{}",
                BoolTargetDisplay { t: result },
                BoolTargetDisplay { t: b1 },
                BoolTargetDisplay { t: b2 },
            ));
        }
        result
    }

    pub fn or(&mut self, b1: BoolTarget, b2: BoolTarget) -> BoolTarget {
        let result = self.builder.or(b1, b2);
        if self.output_enabled() {
            self.handle_output(format!(
                "{} = or {},{}",
                BoolTargetDisplay { t: result },
                BoolTargetDisplay { t: b1 },
                BoolTargetDisplay { t: b2 },
            ));
        }
        result
    }

    pub fn add(&mut self, x: Target, y: Target) -> Target {
        let result = self.builder.add(x, y);
        if self.output_enabled() {
            self.handle_output(format!(
                "{} = add {},{}",
                TargetDisplay { t: result },
                TargetDisplay { t: x },
                TargetDisplay { t: y },
            ));
        }
        result
    }

    pub fn sub(&mut self, x: Target, y: Target) -> Target {
        let result = self.builder.sub(x, y);
        if self.output_enabled() {
            self.handle_output(format!(
                "{} = sub {},{}",
                TargetDisplay { t: result },
                TargetDisplay { t: x },
                TargetDisplay { t: y },
            ));
        }
        result
    }

    pub fn not(&mut self, b: BoolTarget) -> BoolTarget {
        let result = self.builder.not(b);
        if self.output_enabled() {
            self.handle_output(format!(
                "{} = not {}",
                BoolTargetDisplay { t: result },
                BoolTargetDisplay { t: b },
            ));
        }
        result
    }

    pub fn assert_bool(&mut self, b: BoolTarget) {
        self.builder.assert_bool(b);
        if self.output_enabled() {
            self.handle_output(format!("assert_bool {}", BoolTargetDisplay { t: b }));
        }
    }

    pub fn connect(&mut self, x: Target, y: Target) {
        self.builder.connect(x, y);
        if self.output_enabled() {
            self.handle_output(format!(
                "connect {},{}",
                TargetDisplay { t: x },
                TargetDisplay { t: y }
            ));
        }
    }

    pub fn register_public_inputs(&mut self, targets: &[Target]) {
        self.builder.register_public_inputs(targets);
        if self.output_enabled() {
            self.handle_output(format!(
                "register_public_inputs {}",
                TargetSliceDisplay { t: targets }
            ));
        }
    }

    pub fn add_many<T>(&mut self, terms: impl IntoIterator<Item = T> + Clone) -> Target
    where
        T: Borrow<Target>,
    {
        if self.output_enabled() {
            let result = self.builder.add_many(terms.clone());
            self.handle_output(format!(
                "{} = add_many {}",
                TargetDisplay { t: result },
                TargetIntoIteratorDisplay { t: terms },
            ));
            result
        } else {
            self.builder.add_many(terms)
        }
    }

    pub fn le_sum(
        &mut self,
        bits: impl Iterator<Item = impl Borrow<BoolTarget>> + Clone,
    ) -> Target {
        if self.output_enabled() {
            let result = self.builder.le_sum(bits.clone());
            self.handle_output(format!(
                "{} = le_sum {}",
                TargetDisplay { t: result },
                BoolTargetIteratorDisplay { t: bits },
            ));
            result
        } else {
            self.builder.le_sum(bits)
        }
    }

    pub fn range_check(&mut self, x: Target, n_log: usize) {
        self.builder.range_check(x, n_log);
        if self.output_enabled() {
            self.handle_output(format!("range_check\t{},{}", TargetDisplay { t: x }, n_log));
        }
    }

    pub fn add_virtual_bool_target_unsafe(&mut self) -> BoolTarget {
        let result = self.builder.add_virtual_bool_target_unsafe();
        if self.output_enabled() {
            self.handle_output(format!(
                "{} = add_virtual_bool_target_unsafe",
                BoolTargetDisplay { t: result }
            ));
        }
        result
    }

    pub fn add_virtual_bool_target_safe(&mut self) -> BoolTarget {
        let result = self.builder.add_virtual_bool_target_safe();
        if self.output_enabled() {
            self.handle_output(format!(
                "{} = add_virtual_bool_target_safe",
                BoolTargetDisplay { t: result }
            ));
        }
        result
    }

    pub fn constant_u32(&mut self, c: u32) -> U32Target {
        let result = self.builder.constant_u32(c);
        if self.output_enabled() {
            self.handle_output(format!("{} = constant_u32 {}", U32TargetDisplay { t: result }, c));
        }
        result
    }

    pub fn add_u32(&mut self, a: U32Target, b: U32Target) -> (U32Target, U32Target) {
        let result = self.builder.add_u32(a, b);
        if self.output_enabled() {
            self.handle_output(format!(
                "({},{}) = add_u32 {},{}",
                U32TargetDisplay { t: result.0 },
                U32TargetDisplay { t: result.1 },
                U32TargetDisplay { t: a },
                U32TargetDisplay { t: b },
            ));
        }
        result
    }

    pub fn split_le_base<const B: usize>(&mut self, x: Target, num_limbs: usize) -> Vec<Target> {
        let result = self.builder.split_le_base::<B>(x, num_limbs);
        if self.output_enabled() {
            self.handle_output(format!(
                "{} = split_le_base\t{},{},{}",
                VecTargetDisplay { t: &result },
                B,
                TargetDisplay { t: x },
                num_limbs,
            ))
        }
        result
    }

    pub fn add_virtual_target(&mut self) -> Target {
        let result = self.builder.add_virtual_target();
        if self.output_enabled() {
            self.handle_output(format!("{} = add_virtual_target", TargetDisplay { t: result }));
        }
        result
    }

    pub fn comment(&mut self, s: String) {
        if self.output_enabled() {
            self.handle_output(format!("# {}", s));
        }
    }

    pub fn comment_divmod_begin(&mut self, numerator: Target, denominator: Target) {
        self.comment(format!(
            "divmod begin (numerator = {}, denominator = {})",
            TargetDisplay { t: numerator },
            TargetDisplay { t: denominator }
        ));
    }

    pub fn comment_divmod_end(&mut self, quotient: Target, remainder: Target) {
        self.comment(format!(
            "divmod end (quotient = {}, remainder = {})",
            TargetDisplay { t: quotient },
            TargetDisplay { t: remainder }
        ));
    }

    pub fn comment_lessthan_begin(&mut self, a: Target, b: Target, signed: bool) {
        self.comment(format!(
            "lessthan begin (a = {}, b = {}, signed = {})",
            TargetDisplay { t: a },
            TargetDisplay { t: b },
            signed
        ));
    }

    pub fn comment_lessthan_end(&mut self, result: BoolTarget) {
        self.comment(format!("lessthan end (result = {})", BoolTargetDisplay { t: result }));
    }

    fn add_debug_trace_source_file_line(&mut self, name: String, line_number: usize) {
        let name = PathBuf::from_str(&name)
            .unwrap()
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let dtlist = self.debug_trace_list.as_mut().unwrap();

        if let Some(prev_dsp) = &self.prev_source_point {
            if let Some(last_range_vec) = dtlist.source_map.get_mut(prev_dsp) {
                last_range_vec
                    .back_mut()
                    .expect("Range vec for previous source point cannot be empty.")
                    .end = Some(dtlist.list.len() - 1);
            } else {
                panic!("No entry found for the previous plonky2 asm list index range");
            }
        }

        let dsp = SourcePoint { file: name, line_number };
        if let Some(line_list) = dtlist.source_map.get_mut(&dsp) {
            line_list.push_back(AsmListIndexRange { start: dtlist.list.len(), end: None });
        } else {
            let mut deq = VecDeque::new();
            deq.push_back(AsmListIndexRange { start: dtlist.list.len(), end: None });
            dtlist.source_map.insert(dsp.clone(), deq);
        }
        self.prev_source_point = Some(dsp);
    }

    fn comment_source_file_name(&mut self, name: PathString) {
        if self.debug_trace_list.is_none() {
            self.comment(format!("[{}]", name));
        }
    }

    fn comment_source_line(&mut self, line_number: usize, s: &str) {
        if self.debug_trace_list.is_some() {
            let last_file_name = self.file_map.name(self.last_file).unwrap();
            self.add_debug_trace_source_file_line(last_file_name.to_string(), line_number);
        } else {
            self.comment(format!("[{}] {}", line_number, s));
        }
    }

    pub fn comment_update_call_stack(&mut self, call_stack: CallStack) {
        if call_stack != self.last_call_stack {
            if let Some(last_loc) = call_stack.clone().into_iter().last() {
                let span_begin =
                    self.file_map.location(last_loc.file, last_loc.span.start() as usize).unwrap();
                let span_end =
                    self.file_map.location(last_loc.file, last_loc.span.end() as usize).unwrap();

                if last_loc.file != self.last_file {
                    self.comment_source_file_name(self.file_map.name(last_loc.file).unwrap());
                    self.last_file = last_loc.file;
                    self.last_line_number_begin = 0;
                    self.last_line_number_end = 0;
                }
                if (span_begin.line_number != self.last_line_number_begin)
                    || (span_end.line_number != self.last_line_number_end)
                {
                    self.last_line_number_begin = span_begin.line_number;
                    self.last_line_number_end = span_end.line_number;
                    for ln in span_begin.line_number..span_end.line_number + 1 {
                        let lr = self.file_map.line_range(last_loc.file, ln - 1).unwrap();
                        self.comment_source_line(
                            ln,
                            &self.file_map.source(last_loc.file).unwrap()[lr.start..lr.end]
                                .to_string()
                                .trim(),
                        );
                    }
                }
            }
            self.last_call_stack = call_stack;
        }
    }
}
