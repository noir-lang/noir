#!/usr/bin/python3

from collections import namedtuple
try:
    import z3
except ImportError:
    print ("""You need to install z3 for this script to work:
Go to https://github.com/Z3Prover/z3/releases and download the latest *.whl file for your system.
Then install it with python3 -m pip install ./<name>.whl""")
    exit()
from copy import copy
try:
    from colorama import init, Fore, Back, Style
except ImportError:
    print ("""You need to install colorama:
python3 -m pip install colorama""")
    exit()

import re

modulus = 1 # not the actual modulus, just need it to be global
register_width = 64

asm_enabled = False
c_enabled = False

State = namedtuple(
    "State",
    "instructions variables flags solver unused_flags flag_trace trace lines",
)

# Machine state consists of:
# + List of instructions
# + Variable dictionary (for registers and constants)
# + Flag dictionary
# + Z3 Solver instance
# + Unused flag list (needed for ADX without BMI2 case)
# + Flag trace (needed for ADX with BMI2 case)
# + Trace (which variables are used in which instruction)
# + Original code lines
machine_state = State([], {}, {}, z3.Solver(), [], [], {}, {})

# Same variables are used in assembly/c under various names, so we need a dictionary
translation_dict = {}


def refresh_machine_state():
    """Refresh the machine state completely"""
    global machine_state
    machine_state = State([], {}, {}, z3.Solver(), [], [], {}, {})


def add_output_variable_to_trace(variable, instruction_index):
    """"""
    if instruction_index not in machine_state.trace:
        machine_state.trace[instruction_index] = [variable]
    else:
        machine_state.trace[instruction_index] = machine_state.trace[
            instruction_index
        ] + [variable]


def solve():
    """Attempt to solve the system of equation currently saved to instructions"""
    for instruction in machine_state.instructions:
        machine_state.solver.add(instruction)
    return machine_state.solver.check()


def get_input_flag(flag_name):
    """Get the last variable representing this flag, if it doesn't exist, create it"""
    if not flag_name in machine_state.flags:
        z3_variable = z3.BitVec(flag_name + "_" + str(0), 1)
        machine_state.flags[flag_name] = [z3_variable]
        return z3_variable
    else:
        used_flag = machine_state.flags[flag_name][-1]
        if used_flag in machine_state.unused_flags:
            machine_state.unused_flags.remove(used_flag)
        return machine_state.flags[flag_name][-1]


def get_output_flag(flag_name):
    """Create a new variable representing this flag"""
    if not flag_name in machine_state.flags:
        z3_variable = z3.BitVec(flag_name + "_" + str(0), 1)
        machine_state.flags[flag_name] = [z3_variable]
        machine_state.unused_flags.append(z3_variable)
        return z3_variable
    else:
        z3_variable = z3.BitVec(
            flag_name + "_" + str(len(machine_state.flags[flag_name])), 1
        )
        machine_state.flags[flag_name].append(z3_variable)

        machine_state.unused_flags.append(z3_variable)
        return z3_variable


def get_input_variable(var_name):
    """Get the last symbolic variable representing this variable"""
    global translation_dict
    if var_name in translation_dict:
        var_name = translation_dict[var_name]
    if not var_name in machine_state.variables:
        z3_variable = z3.BitVec(var_name + "_" + str(0), register_width)
        machine_state.variables[var_name] = [z3_variable]
        return z3_variable
    else:
        return machine_state.variables[var_name][-1]


def get_output_variable(var_name):
    """Generate a new symbolic variable"""
    global translation_dict

    if var_name in translation_dict:
        var_name = translation_dict[var_name]

    if not var_name in machine_state.variables:
        z3_variable = z3.BitVec(var_name + "_" + str(0), register_width)
        machine_state.variables[var_name] = [z3_variable]

        return z3_variable
    else:
        z3_variable = z3.BitVec(
            var_name + "_" + str(len(machine_state.variables[var_name])), register_width
        )
        machine_state.variables[var_name].append(z3_variable)

        return z3_variable


def set_input_variable(var_name, value):
    """Set the value of a symbolic variable"""
    global translation_dict
    if var_name in translation_dict:
        var_name = translation_dict[var_name]
    if var_name in machine_state.variables:
        inp_var = machine_state[var_name][0]
    else:
        inp_var = get_input_variable(var_name)
    machine_state.instructions.append(inp_var == z3.BitVecVal(value, 64))


def get_output_variable_value(var_name):
    """Retrieve the value of the variable from the model"""
    return machine_state.solver.model()[get_input_variable(var_name)].as_long()


# Translation functions:

def generate_translator():
    """Fill the global translation dictionary for translating assembly variables and registers into symbolic names we are going to use"""
    global translation_dict
    translation_dict["T::r_inv"] = "r_inv"
    translation_dict["%[r_inv]"] = "r_inv"
    for i in range(4):
        translation_dict[f"data[{i}]"] = f"this[{i}]"
        translation_dict[f"other.data[{i}]"] = f"other[{i}]"
        translation_dict[f"modulus.data[{i}]"] = f"modulus[{i}]"
        translation_dict[f"%[modulus_{i}]"] = f"modulus[{i}]"
        translation_dict[f'{i*8}(" b ")'] = f"other[{i}]"
        translation_dict[f'{i*8}(" a ")'] = f"other[{i}]"
        translation_dict['" a' + str(i + 1) + ' "'] = f"this[{i}]"
    registers = [
        "rdx",
        "rdi",
        "r8",
        "r9",
        "r10",
        "r11",
        "r12",
        "r13",
        "r14",
        "r15",
    ]
    for register in registers:
        translation_dict[r"%%" + register] = "reg_" + register
    asm_inp_vars = ["a1", "a2", "a3", "a4"]
    translation_dict[r"%[zero_reference]"] = "zero"


def get_asm_input(arg):
    """Parse the assembly input register or constant and create a z3 variable or constant accordingly"""
    if arg[0] == "$":
        return z3.BitVecVal(int(arg[1:]), register_width)
    else:
        return get_input_variable(arg)


def movq(args):
    """Construct a movq instruction constraint"""
    var_in = get_asm_input(args[0])
    var_out = get_output_variable(args[1])
    add_output_variable_to_trace(var_out, len(machine_state.instructions))
    machine_state.instructions.append(var_in == var_out)


def xorq(args):
    """Construct a xorq instruction constraint"""
    var_in_1 = get_asm_input(args[0])
    var_in_2 = get_asm_input(args[1])
    CF_flag = get_output_flag("CF")
    OF_flag = get_output_flag("OF")
    var_out = get_output_variable(args[1])
    add_output_variable_to_trace(var_out, len(machine_state.instructions))
    machine_state.instructions.append((var_in_1 ^ var_in_2) == var_out)
    machine_state.instructions.append(CF_flag == 0)
    machine_state.instructions.append(OF_flag == 0)


def mulxq(args):
    """Construct a mulxq instruction constraint"""
    var_in_1 = get_asm_input("reg_rdx")
    var_in_2 = get_asm_input(args[0])
    var_out_1 = get_output_variable(args[1])
    var_out_2 = get_output_variable(args[2])
    add_output_variable_to_trace(var_out_1, len(machine_state.instructions))
    add_output_variable_to_trace(var_out_2, len(machine_state.instructions))
    temp = z3.ZeroExt(register_width, var_in_1) * z3.ZeroExt(register_width, var_in_2)
    machine_state.instructions.append(
        var_out_1 == z3.Extract(register_width - 1, 0, temp)
    )
    machine_state.instructions.append(
        var_out_2 == z3.Extract(2 * register_width - 1, register_width, temp)
    )


def addq(args):
    """Construct an addq instruction constraint"""
    var_in_1 = get_asm_input(args[0])
    var_in_2 = get_asm_input(args[1])
    var_out = get_output_variable(args[1])
    CF_out = get_output_flag("CF")
    temp = z3.ZeroExt(1, var_in_1) + z3.ZeroExt(1, var_in_2)
    add_output_variable_to_trace(var_out, len(machine_state.instructions))
    machine_state.instructions.append(
        var_out == z3.Extract(register_width - 1, 0, temp)
    )
    machine_state.instructions.append(
        CF_out == z3.Extract(register_width, register_width, temp)
    )


def adcq(args):
    """Construct an adcq instruction constraint"""
    global instruction_index
    var_in_1 = get_asm_input(args[0])
    var_in_2 = get_asm_input(args[1])
    var_out = get_output_variable(args[1])

    CF_in = get_input_flag("CF")
    CF_out = get_output_flag("CF")
    machine_state.flag_trace.append((instruction_index, CF_out, CF_in, var_out))
    add_output_variable_to_trace(var_out, len(machine_state.instructions))
    temp = z3.ZeroExt(1, var_in_1) + z3.ZeroExt(1, var_in_2) + z3.ZeroExt(64, CF_in)
    machine_state.instructions.append(
        var_out == z3.Extract(register_width - 1, 0, temp)
    )
    machine_state.instructions.append(
        CF_out == z3.Extract(register_width, register_width, temp)
    )


def adxoq(args):
    """Construct an adxoq instruction constraint"""
    global instruction_index
    var_in_1 = get_asm_input(args[0])
    var_in_2 = get_asm_input(args[1])
    var_out = get_output_variable(args[1])

    OF_in = get_input_flag("OF")
    OF_out = get_output_flag("OF")
    add_output_variable_to_trace(var_out, len(machine_state.instructions))
    machine_state.flag_trace.append((instruction_index, OF_out, OF_in, var_out))
    temp = z3.ZeroExt(1, var_in_1) + z3.ZeroExt(1, var_in_2) + z3.ZeroExt(64, OF_in)
    machine_state.instructions.append(
        var_out == z3.Extract(register_width - 1, 0, temp)
    )
    machine_state.instructions.append(
        OF_out == z3.Extract(register_width, register_width, temp)
    )


instruction_index = 0


def parse_asm_line(line):
    """Parse one line of assembly from asm_macros.hpp and convert it to instruction constraints"""
    global instruction_index
    opcode = line[: line.find(" ")]
    args_part = line[line.find(" ") + 1 :]
    pre_args = [pre_arg.strip() for pre_arg in args_part.split(",")]
    machine_state.lines[len(machine_state.instructions)] = line
    if opcode == "movq":
        movq(pre_args)
    elif opcode == "xorq":
        xorq(pre_args)
    elif opcode == "mulxq":
        mulxq(pre_args)
    elif opcode == "addq":
        addq(pre_args)
    elif opcode == "adcq":
        adcq(pre_args)
    elif opcode == "adcxq":
        adcq(pre_args)
    elif opcode == "adoxq":
        adxoq(pre_args)
    else:
        raise Exception("Unknown opcode: " + opcode + "in line: " + line)
    instruction_index += 1


def parse_asm_macros_hpp(filename, chosen_def=0):
    """Parse the assembly to create symbolic constraints. chosen_def=0 for non-BMI, 1 for BMI"""
    global asm_enabled
    asm_enabled = True
    lines = []
    # Read the lines
    with open(filename) as f:
        lines = [line for line in f]

    # Get the lines in the first SQR definition (these are BMI lines)
    start = -1
    for (i, line) in enumerate(lines):

        if start != -1:
            if line.find("#define") == 0:
                end = i
                break

        if start == -1:
            if line.find("#define SQR(a)") == 0:
                if chosen_def == 0:
                    start = i
                else:
                    chosen_def -= 1

    # We want to go back, since there are several comments we've picked up
    while end != 0:
        if len(lines[end].strip()) == 0:
            break
        end -= 1

    # Trim the lines
    lines_trimmed = [
        line[: line.find("\\n\\t")].strip()[1:]
        for line in lines[start:end]
        if line.find('"') != -1 and line.find("Partial fix")==-1
    ]

    # Parse the lines to create rules
    for line in lines_trimmed:
        print(f"{Fore.RED}{line}{Fore.GREEN}")
        start = len(machine_state.instructions)
        parse_asm_line(line)
        end = len(machine_state.instructions)
        for i in range(start, end):
            print("\t", machine_state.instructions[i])
        print(f"{Style.RESET_ALL}")

def fill_starting_vars():
    """Fill the global constants used in montgomery multiplication, such as the modulus, r squared, r and r_inv. Also, restrain the input variables to be less than the modulus"""
    global modulus
    bn_p = (
        (0x30644E72E131A029 << (64 * 3))
        + (0xB85045B68181585D << (128))
        + (0x97816A916871CA8D << 64)
        + 0x3C208C16D87CFD47
    )  # Fq
    modulus = bn_p

    bn_p_r_inv = 0x87D20782E4866389  # Fq

    wide_vars = [("modulus", bn_p)]

    # Restrain the variables to be less than the doubled modulus since we are using lazy reductions
    restrained_vars = ["other", "this"]
    for restrained_var in restrained_vars:
        big_var = z3.BitVec(restrained_var + "_huge", register_width * 4)
        machine_state.instructions.append(
            z3.ULT(big_var, z3.BitVecVal(2 * bn_p, register_width * 4))
        )
        for i in range(4):

            vvar = get_input_variable(restrained_var + f"[{i}]")
            machine_state.instructions.append(
                vvar
                == z3.Extract(
                    register_width * (i + 1) - 1, register_width * (i), big_var
                )
            )
    # Save the modulus and its parts
    for (w_var, w_var_val) in wide_vars:
        for i in range(4):
            vvar = get_input_variable(w_var + f"[{i}]")
            machine_state.instructions.append(
                vvar
                == z3.BitVecVal(
                    (w_var_val >> (64 * i)) & ((1 << 64) - 1),
                    register_width,
                )
            )

    r_inv_var = get_input_variable("r_inv")
    machine_state.instructions.append(r_inv_var == bn_p_r_inv)
    zero_var = get_input_variable("zero")
    machine_state.instructions.append(zero_var == 0)


def print_outputs():
    """Print the values of variables in the model"""
    global asm_enabled
    if asm_enabled:
        t0_asm = get_output_variable_value("reg_r12")
        t1_asm = get_output_variable_value("reg_r13")
        t2_asm = get_output_variable_value("reg_r14")
        t3_asm = get_output_variable_value("reg_r15")

        ar_asm = [t3_asm, t2_asm, t1_asm, t0_asm]
        acc_asm = 0
        for x in ar_asm:
            acc_asm = acc_asm << 64
            acc_asm += x
        print(hex(acc_asm))

    print("Inputs:")
    print("Other:")
    other_full = machine_state.solver.model()[
        z3.BitVec("other_huge", register_width * 4)
    ].as_long()

    print("Long format: ", hex(other_full), other_full < modulus)
    print("Limbs:")
    for i in range(4):
        print(hex(get_output_variable_value(f"other[{i}]")))

    print("This:")
    this_full = machine_state.solver.model()[
        z3.BitVec("this_huge", register_width * 4)
    ].as_long()

    print("Long format: ", hex(this_full), this_full < modulus)

    print("Limbs:")

    for i in range(4):
        print(hex(get_output_variable_value(f"this[{i}]")))


def print_trace():
    instruction_indices = list(machine_state.lines.keys())
    instruction_indices.sort()
    for (i, index) in enumerate(instruction_indices):
        if i == (len(instruction_indices) - 1):
            end = len(machine_state.instructions)
        else:
            end = instruction_indices[i + 1]
        print(f"{Fore.BLUE}{machine_state.lines[index]}")
        for j in range(index, end):
            print(f"{Fore.GREEN}{machine_state.instructions[j]}")
        for v in machine_state.trace[index]:
            print(
                f"{Fore.CYAN}{v}{Fore.WHITE} == {Fore.RED}{hex(machine_state.solver.model()[v].as_long())}"
            )


def solve_for_unused_flags(trim_first=0):
    """Check if dangling flags (the case of non-BMI2 ADX) can be set to 1, causing errors"""
    
    # Add instructions to the solver
    for instruction in machine_state.instructions:
        machine_state.solver.add(instruction)

    current_timeout = 0 # No timeout. Try to solve forever or until we run out of memory and are killed by the mainframe
    
    # We can skip flags. Makes it easier to run a parallel script if we're stuck
    chosen_flags = machine_state.unused_flags[trim_first:]
    # Print the set of flags to be analyzed
    print(chosen_flags)
    print ("Sat means an error-inducing input has been found.")
    print ("Unsat means that no such input exists")
    print ("Unknown means that the solver didn't come to a conclusion in the given time")

    z3.set_param("timeout", current_timeout)
    solution_results = dict()
    unsolved_flags = copy(chosen_flags)

    # Do until we've solved all
    while len(unsolved_flags) != 0:

        z3.set_param("timeout", current_timeout)
        temp_unsolved = copy(unsolved_flags)
        temp_solved = []
        # For each flags
        for (i, flag) in enumerate(temp_unsolved):
            print ("Solving for",flag)
            # Save the machine state with all the instructions
            machine_state.solver.push()
            # Set the flag to 1
            machine_state.solver.add(flag == 1)
            # Set timeout
            machine_state.solver.set("timeout", current_timeout)
            # Execute the solver
            result = machine_state.solver.check()
            # Print result (sat, unsat, unknown)
            print(f"{i+1}/{len(temp_unsolved)}", flag, result)
            # If not unknown, then we are either sure that it can't be set or found a bug
            if result != z3.unknown:
                temp_solved.append(flag)
                solution_results[flag] = result
            
            # If we haven't found a bug, go to next
            if result != z3.sat:

                machine_state.solver.pop()
                continue
            
            # If there is a bug, print solution
            print_outputs()
            machine_state.solver.pop()

        # Remove those that were solved
        for x in temp_solved:
            unsolved_flags.remove(x)
        # Double the timeout
        current_timeout *= 2
        # Print what's been found by now
        for flag in solution_results.keys():
            print(flag, solution_results[flag])
        # Print what's left to do
        print("Flags left to solve:", len(unsolved_flags))


def solve_for_given_flags(given_flags, trim_first=0):
    """Check if a given set of flags can be 1"""
    # Add instructions to the solver
    for instruction in machine_state.instructions:
        machine_state.solver.add(instruction)

    # Set timeout here if you want to check something quickly. 0 means no timeout
    current_timeout = 0
    # You can skip some flags (useful for running a second script starting from a flag after the solver gets stuck on one)
    chosen_flags = given_flags[trim_first:]
    print ("Flags we'll check:")
    print(chosen_flags)
    print ("Sat means an error-inducing input has been found.")
    print ("Unsat means that no such input exists")
    print ("Unknown means that the solver didn't come to a conclusion in the given time")
    z3.set_param("timeout", current_timeout)
    
    solution_results = dict()
    unsolved_flags = copy(chosen_flags)
    # Do till we've solved all flags
    while len(unsolved_flags) != 0:

        z3.set_param("timeout", current_timeout)
        temp_unsolved = copy(unsolved_flags)
        temp_solved = []
        # For each flags
        for (i, flag) in enumerate(temp_unsolved):
            print ('Solving for',flag)
            # Save the solver state with other instructions
            machine_state.solver.push()
            # Add the flag constraint
            machine_state.solver.add(flag == 1)
            machine_state.solver.set("timeout", current_timeout)
            # Execute the solver
            result = machine_state.solver.check()
            # Print result (sat, unsat, unknown)
            print(f"{i+1}/{len(temp_unsolved)}", flag, result)
            # If not unknown, remeber the result
            if result != z3.unknown:
                temp_solved.append(flag)
                solution_results[flag] = result
            # If not a bug, go to next
            if result != z3.sat:

                machine_state.solver.pop()
                continue
            
            # If a bug, print failing input
            print_outputs()
            machine_state.solver.pop()

        for x in temp_solved:
            unsolved_flags.remove(x)
        current_timeout *= 2
        for flag in solution_results.keys():
            print(flag, solution_results[flag])
        print("Flags left to solve:", len(unsolved_flags))


def solve_for_single_flag(flag_num):
    """Solve for a particular flag from unused flags"""
    for instruction in machine_state.instructions:
        machine_state.solver.add(instruction)

    chosen_flag = machine_state.unused_flags[flag_num]

    print("Solving SMT for flag ", chosen_flag)
    machine_state.solver.add(chosen_flag == 1)
    machine_state.solver.set("timeout", 0)
    result = machine_state.solver.check()

    if result != z3.unknown:
        if result != z3.sat:

            raise RuntimeError("Only flags that can be solved should be used")
    else:
        raise RuntimeError("Only flags that can be solved should be used")
    # print_outputs()
    return machine_state.solver.model()[
        z3.BitVec("other_huge", register_width * 4)
    ].as_long()

def prohibit_value(name, width, value):
    """Prohibit a particular variable from becoming a particualr value"""
    machine_state.instructions.append(z3.BitVec(name, width) != value)


def choose_value(name, width, value):
    """Equate a variable to a value. Can be used to quickly check a value of a particular register after giving a particular input to the function"""
    machine_state.instructions.append(z3.BitVec(name, width) == value)


def solve_for_result():
    """Old function, which I used to find solutions to getting a particular end value"""
    for instruction in machine_state.instructions:
        machine_state.solver.add(instruction)
    register_values = [
        ("reg_r12", 9369308295397661479),
        ("reg_r13", 2866003295495262720),
        ("reg_r14", 2323916859244912136),
        ("reg_r15", 17784720819236159745),
    ]
    for (a, b) in register_values:
        machine_state.solver.add(
            get_output_variable(a) == z3.BitVecVal(b, register_width)
        )

    result = machine_state.solver.check()
    print(result)
    if result == z3.sat:
        print_outputs()


def solve():
    """Solve the system of equations and print the output"""
    for instruction in machine_state.instructions:
        machine_state.solver.add(instruction)

    result = machine_state.solver.check()

    print(result)
    if result == z3.sat:
        print_outputs()


def print_instructions():
    """Print instructions"""
    for instruction in machine_state.instructions:
        print(instruction)


def get_downshifting_flags():
    """Get the flags, which are added to a register with a lower index than from which they spawned"""
    last_state = {}
    collected_flags = []
    for (instruction_index, out_flag, in_flag, out_reg) in machine_state.flag_trace:
        flag_type = str(out_flag)[0 : str(out_flag).find("_")]
        reg_index = re.search(r"(?<=reg_r)\d+(?=_\d+)", str(out_reg)).group(0)
        if flag_type not in last_state:
            last_state[flag_type] = (reg_index, out_flag, out_reg)
        else:
            prev_state = last_state[flag_type]
            last_state[flag_type] = (reg_index, out_flag, out_reg)
            if int(prev_state[0]) > int(reg_index):
                print(
                    "Found a flag that needs checking: ",
                    prev_state[0],
                    reg_index,
                    prev_state[1],
                    prev_state[2],
                    out_reg,
                )
                collected_flags.append(prev_state[1])

    return collected_flags


if __name__ == "__main__":
    init() # init colorama
    prohibited_values = []
    # z3.set_param("parallel.enable", True)
    # z3.set_param("parallel.threads.max", 48)
    refresh_machine_state()
    generate_translator()
    fill_starting_vars()
    # Get the BMI version
    parse_asm_macros_hpp("asm_macros.hpp", 1)
    solve_for_given_flags(get_downshifting_flags())
    # Get the non-BMI version
    #parse_asm_macros_hpp("asm_macros.hpp",0)
    #solve_for_unused_flags()
