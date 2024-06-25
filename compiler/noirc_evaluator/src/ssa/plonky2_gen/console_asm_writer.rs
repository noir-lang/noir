use std::fmt::Write;

use plonky2::iop::{
    target::{BoolTarget, Target},
    wire::Wire,
};

use super::config::P2Field;
use super::{asm_writer::AsmWriter, config::P2Builder};

pub(crate) struct ConsoleAsmWriter {
    pub builder: P2Builder,
    pub show_plonky2: bool,
}

fn target2string(t: Target) -> String {
    match t {
        Target::VirtualTarget { index } => {
            format!("v{}", index)
        }
        Target::Wire(Wire { row, column }) => {
            format!("r{}c{}", row, column)
        }
    }
}

fn booltarget2string(t: &BoolTarget) -> String {
    target2string(t.target)
}

fn vecbooltarget2string(t: &Vec<BoolTarget>) -> String {
    let mut result = String::new();
    write!(&mut result, "(");
    let mut first = true;
    for bt in t {
        if first {
            write!(&mut result, "{}", booltarget2string(bt));
            first = false;
        } else {
            write!(&mut result, ",{}", booltarget2string(bt));
        }
    }
    write!(&mut result, ")");
    result
}

fn targetslice2string(t: &[Target]) -> String {
    let mut result = String::new();
    write!(&mut result, "(");
    let mut first = true;
    for tt in t {
        if first {
            write!(&mut result, "{}", target2string(*tt));
            first = false;
        } else {
            write!(&mut result, ",{}", target2string(*tt));
        }
    }
    write!(&mut result, ")");
    result

}

impl AsmWriter for ConsoleAsmWriter {
    fn get_builder(&self) -> &P2Builder {
        &self.builder
    }
    fn get_mut_builder(&mut self) -> &mut P2Builder {
        &mut self.builder
    }
    fn move_builder(self) -> P2Builder {
        self.builder
    }

    fn new(builder: P2Builder, show_plonky2: bool) -> Self {
        ConsoleAsmWriter { builder, show_plonky2 }
    }

    fn is_equal(&mut self, x: Target, y: Target) -> BoolTarget {
        let result = self.builder.is_equal(x, y);
        if self.show_plonky2 {
            println!(
                "is_equal\t{},{},{}",
                target2string(x),
                target2string(y),
                booltarget2string(&result)
            );
        }
        result
    }

    fn zero(&mut self) -> Target {
        let result = self.builder.zero();
        if self.show_plonky2 {
            println!("zero\t{}", target2string(result));
        }
        result
    }

    fn one(&mut self) -> Target {
        let result = self.builder.one();
        if self.show_plonky2 {
            println!("one\t{}", target2string(result));
        }
        result
    }

    fn two(&mut self) -> Target {
        let result = self.builder.two();
        if self.show_plonky2 {
            println!("two\t{}", target2string(result));
        }
        result
    }

    fn split_le(&mut self, integer: Target, num_bits: usize) -> Vec<BoolTarget> {
        let result = self.builder.split_le(integer, num_bits);
        if self.show_plonky2 {
            println!(
                "split_le\t{},{},{}",
                target2string(integer),
                num_bits,
                vecbooltarget2string(&result)
            );
        }
        result
    }

    fn _if(&mut self, b: BoolTarget, x: Target, y: Target) -> Target {
        let result = self.builder._if(b, x, y);
        if self.show_plonky2 {
            println!(
                "if\t{},{},{},{}",
                booltarget2string(&b),
                target2string(x),
                target2string(y),
                target2string(result)
            );
        }
        result
    }

    fn exp_u64(&mut self, base: Target, exponent: u64) -> Target {
        let result = self.builder.exp_u64(base, exponent);
        if self.show_plonky2 {
            println!("exp_u64\t{},{},{}", target2string(base), exponent, target2string(result));
        }
        result
    }

    fn constant(&mut self, c: P2Field) -> Target {
        let result = self.builder.constant(c);
        if self.show_plonky2 {
            println!("constant\t{},{}", c, target2string(result));
        }
        result
    }

    fn constant_bool(&mut self, b: bool) -> BoolTarget {
        let result = self.builder.constant_bool(b);
        if self.show_plonky2 {
            println!("constant_bool\t{},{}", b, booltarget2string(&result));
        }
        result
    }

    fn mul(&mut self, x: Target, y: Target) -> Target {
        let result = self.builder.mul(x, y);
        if self.show_plonky2 {
            println!("mul\t{},{},{}", target2string(x), target2string(y), target2string(result));
        }
        result
    }

    fn and(&mut self, b1: BoolTarget, b2: BoolTarget) -> BoolTarget {
        let result = self.builder.and(b1, b2);
        if self.show_plonky2 {
            println!(
                "and\t{},{},{}",
                booltarget2string(&b1),
                booltarget2string(&b2),
                booltarget2string(&result)
            );
        }
        result
    }

    fn or(&mut self, b1: BoolTarget, b2: BoolTarget) -> BoolTarget {
        let result = self.builder.or(b1, b2);
        if self.show_plonky2 {
            println!(
                "or\t{},{},{}",
                booltarget2string(&b1),
                booltarget2string(&b2),
                booltarget2string(&result)
            );
        }
        result
    }

    fn add(&mut self, x: Target, y: Target) -> Target {
        let result = self.builder.add(x, y);
        if self.show_plonky2 {
            println!("add\t{},{},{}", target2string(x), target2string(y), target2string(result));
        }
        result
    }

    fn sub(&mut self, x: Target, y: Target) -> Target {
        let result = self.builder.sub(x, y);
        if self.show_plonky2 {
            println!("sub\t{},{},{}", target2string(x), target2string(y), target2string(result));
        }
        result
    }

    fn not(&mut self, b: BoolTarget) -> BoolTarget {
        let result = self.builder.not(b);
        if self.show_plonky2 {
            println!("not\t{},{}", booltarget2string(&b), booltarget2string(&result));
        }
        result
    }

    fn assert_bool(&mut self, b: BoolTarget) {
        self.builder.assert_bool(b);
        if self.show_plonky2 {
            println!("assert_bool\t{}", booltarget2string(&b));
        }
    }

    fn connect(&mut self, x: Target, y: Target) {
        self.builder.connect(x, y);
        if self.show_plonky2 {
            println!("connect\t{},{}", target2string(x), target2string(y));
        }
    }

    fn register_public_inputs(&mut self, targets: &[Target]) {
        self.builder.register_public_inputs(targets);
        if self.show_plonky2 {
            println!("register_public_inputs\t{}", targetslice2string(targets));
        }
    }
}
