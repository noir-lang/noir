use std::{
    borrow::Borrow,
    fmt::Display,
    fs::File,
    io::{BufWriter, Write},
};

use plonky2::iop::{
    target::{BoolTarget, Target},
    wire::Wire,
};
use plonky2_u32::gadgets::arithmetic_u32::{CircuitBuilderU32, U32Target};

use super::config::P2Field;
use super::{asm_writer::AsmWriter, config::P2Builder};

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

pub(crate) struct ConsoleAndFileAsmWriter {
    pub builder: P2Builder,
    pub show_plonky2: bool,
    file: Option<BufWriter<File>>,
}

impl ConsoleAndFileAsmWriter {
    fn output_enabled(&self) -> bool {
        self.show_plonky2 || self.file.is_some()
    }

    fn handle_output(&mut self, s: String) {
        if self.show_plonky2 {
            println!("{}", s);
        }

        if let Some(f) = &mut self.file {
            writeln!(f, "{}", s).expect("Unable to write PLONKY2 data to file");
        }
    }
}

impl AsmWriter for ConsoleAndFileAsmWriter {
    fn get_builder(&self) -> &P2Builder {
        &self.builder
    }
    fn get_mut_builder(&mut self) -> &mut P2Builder {
        &mut self.builder
    }
    fn move_builder(self) -> P2Builder {
        self.builder
    }

    fn new(builder: P2Builder, show_plonky2: bool, plonky2_print_file: Option<String>) -> Self {
        ConsoleAndFileAsmWriter {
            builder,
            show_plonky2,
            file: if let Some(file_name) = plonky2_print_file {
                Some(BufWriter::new(
                    File::create(file_name).expect("Unable to create PLONKY2 file"),
                ))
            } else {
                None
            },
        }
    }

    fn is_equal(&mut self, x: Target, y: Target) -> BoolTarget {
        let result = self.builder.is_equal(x, y);
        if self.output_enabled() {
            self.handle_output(format!(
                "is_equal\t{},{},{}",
                TargetDisplay { t: x },
                TargetDisplay { t: y },
                BoolTargetDisplay { t: result },
            ));
        }
        result
    }

    fn zero(&mut self) -> Target {
        let result = self.builder.zero();
        if self.output_enabled() {
            self.handle_output(format!("zero\t{}", TargetDisplay { t: result }));
        }
        result
    }

    fn one(&mut self) -> Target {
        let result = self.builder.one();
        if self.output_enabled() {
            self.handle_output(format!("one\t{}", TargetDisplay { t: result }));
        }
        result
    }

    fn two(&mut self) -> Target {
        let result = self.builder.two();
        if self.output_enabled() {
            self.handle_output(format!("two\t{}", TargetDisplay { t: result }));
        }
        result
    }

    fn split_le(&mut self, integer: Target, num_bits: usize) -> Vec<BoolTarget> {
        let result = self.builder.split_le(integer, num_bits);
        if self.output_enabled() {
            self.handle_output(format!(
                "split_le\t{},{},{}",
                TargetDisplay { t: integer },
                num_bits,
                VecBoolTargetDisplay { t: &result }
            ));
        }
        result
    }

    fn _if(&mut self, b: BoolTarget, x: Target, y: Target) -> Target {
        let result = self.builder._if(b, x, y);
        if self.output_enabled() {
            self.handle_output(format!(
                "if\t{},{},{},{}",
                BoolTargetDisplay { t: b },
                TargetDisplay { t: x },
                TargetDisplay { t: y },
                TargetDisplay { t: result }
            ));
        }
        result
    }

    fn exp_u64(&mut self, base: Target, exponent: u64) -> Target {
        let result = self.builder.exp_u64(base, exponent);
        if self.output_enabled() {
            self.handle_output(format!(
                "exp_u64\t{},{},{}",
                TargetDisplay { t: base },
                exponent,
                TargetDisplay { t: result }
            ));
        }
        result
    }

    fn constant(&mut self, c: P2Field) -> Target {
        let result = self.builder.constant(c);
        if self.output_enabled() {
            self.handle_output(format!("constant\t{},{}", c, TargetDisplay { t: result }));
        }
        result
    }

    fn constant_bool(&mut self, b: bool) -> BoolTarget {
        let result = self.builder.constant_bool(b);
        if self.output_enabled() {
            self.handle_output(format!("constant_bool\t{},{}", b, BoolTargetDisplay { t: result }));
        }
        result
    }

    fn mul(&mut self, x: Target, y: Target) -> Target {
        let result = self.builder.mul(x, y);
        if self.output_enabled() {
            self.handle_output(format!(
                "mul\t{},{},{}",
                TargetDisplay { t: x },
                TargetDisplay { t: y },
                TargetDisplay { t: result }
            ));
        }
        result
    }

    fn and(&mut self, b1: BoolTarget, b2: BoolTarget) -> BoolTarget {
        let result = self.builder.and(b1, b2);
        if self.output_enabled() {
            self.handle_output(format!(
                "and\t{},{},{}",
                BoolTargetDisplay { t: b1 },
                BoolTargetDisplay { t: b2 },
                BoolTargetDisplay { t: result }
            ));
        }
        result
    }

    fn or(&mut self, b1: BoolTarget, b2: BoolTarget) -> BoolTarget {
        let result = self.builder.or(b1, b2);
        if self.output_enabled() {
            self.handle_output(format!(
                "or\t{},{},{}",
                BoolTargetDisplay { t: b1 },
                BoolTargetDisplay { t: b2 },
                BoolTargetDisplay { t: result }
            ));
        }
        result
    }

    fn add(&mut self, x: Target, y: Target) -> Target {
        let result = self.builder.add(x, y);
        if self.output_enabled() {
            self.handle_output(format!(
                "add\t{},{},{}",
                TargetDisplay { t: x },
                TargetDisplay { t: y },
                TargetDisplay { t: result }
            ));
        }
        result
    }

    fn sub(&mut self, x: Target, y: Target) -> Target {
        let result = self.builder.sub(x, y);
        if self.output_enabled() {
            self.handle_output(format!(
                "sub\t{},{},{}",
                TargetDisplay { t: x },
                TargetDisplay { t: y },
                TargetDisplay { t: result }
            ));
        }
        result
    }

    fn not(&mut self, b: BoolTarget) -> BoolTarget {
        let result = self.builder.not(b);
        if self.output_enabled() {
            self.handle_output(format!(
                "not\t{},{}",
                BoolTargetDisplay { t: b },
                BoolTargetDisplay { t: result }
            ));
        }
        result
    }

    fn assert_bool(&mut self, b: BoolTarget) {
        self.builder.assert_bool(b);
        if self.output_enabled() {
            self.handle_output(format!("assert_bool\t{}", BoolTargetDisplay { t: b }));
        }
    }

    fn connect(&mut self, x: Target, y: Target) {
        self.builder.connect(x, y);
        if self.output_enabled() {
            self.handle_output(format!(
                "connect\t{},{}",
                TargetDisplay { t: x },
                TargetDisplay { t: y }
            ));
        }
    }

    fn register_public_inputs(&mut self, targets: &[Target]) {
        self.builder.register_public_inputs(targets);
        if self.output_enabled() {
            self.handle_output(format!(
                "register_public_inputs\t{}",
                TargetSliceDisplay { t: targets }
            ));
        }
    }

    fn add_many<T>(&mut self, terms: impl IntoIterator<Item = T> + Clone) -> Target
    where
        T: Borrow<Target>,
    {
        if self.output_enabled() {
            let result = self.builder.add_many(terms.clone());
            self.handle_output(format!(
                "add_many\t{},{}",
                TargetIntoIteratorDisplay { t: terms },
                TargetDisplay { t: result }
            ));
            result
        } else {
            self.builder.add_many(terms)
        }
    }

    fn le_sum(&mut self, bits: impl Iterator<Item = impl Borrow<BoolTarget>> + Clone) -> Target {
        if self.output_enabled() {
            let result = self.builder.le_sum(bits.clone());
            self.handle_output(format!(
                "le_sum\t{},{}",
                BoolTargetIteratorDisplay { t: bits },
                TargetDisplay { t: result }
            ));
            result
        } else {
            self.builder.le_sum(bits)
        }
    }

    fn range_check(&mut self, x: Target, n_log: usize) {
        self.builder.range_check(x, n_log);
        if self.output_enabled() {
            self.handle_output(format!("range_check\t{},{}", TargetDisplay { t: x }, n_log));
        }
    }

    fn add_virtual_bool_target_unsafe(&mut self) -> BoolTarget {
        let result = self.builder.add_virtual_bool_target_unsafe();
        if self.output_enabled() {
            self.handle_output(format!(
                "add_virtual_bool_target_unsafe\t{}",
                BoolTargetDisplay { t: result }
            ));
        }
        result
    }

    fn add_virtual_bool_target_safe(&mut self) -> BoolTarget {
        let result = self.builder.add_virtual_bool_target_safe();
        if self.output_enabled() {
            self.handle_output(format!(
                "add_virtual_bool_target_safe\t{}",
                BoolTargetDisplay { t: result }
            ));
        }
        result
    }

    fn constant_u32(&mut self, c: u32) -> U32Target {
        let result = self.builder.constant_u32(c);
        if self.output_enabled() {
            self.handle_output(format!("constant_u32\t{},{}", c, U32TargetDisplay { t: result }));
        }
        result
    }

    fn add_u32(&mut self, a: U32Target, b: U32Target) -> (U32Target, U32Target) {
        let result = self.builder.add_u32(a, b);
        if self.output_enabled() {
            self.handle_output(format!(
                "add_u32\t{},{},({},{})",
                U32TargetDisplay { t: a },
                U32TargetDisplay { t: b },
                U32TargetDisplay { t: result.0 },
                U32TargetDisplay { t: result.1 }
            ));
        }
        result
    }

    fn split_le_base<const B: usize>(&mut self, x: Target, num_limbs: usize) -> Vec<Target> {
        let result = self.builder.split_le_base::<B>(x, num_limbs);
        if self.output_enabled() {
            self.handle_output(format!(
                "split_le_base\t{},{},{},{}",
                B,
                TargetDisplay { t: x },
                num_limbs,
                VecTargetDisplay { t: &result }
            ))
        }
        result
    }

    fn add_virtual_target(&mut self) -> Target {
        let result = self.builder.add_virtual_target();
        if self.output_enabled() {
            self.handle_output(format!("add_virtual_target\t{}", TargetDisplay { t: result }));
        }
        result
    }
}
