# VM Control Flow + High-level architecture

This document breaks down the VM into internal components and defines how these components interact with one another.

This can be considered an intermediate abstraction layer between the high-level VM definition and the explicit circuit architecture. The intention is for this abstraction to be analogous to how CPU chips are broken down into discrete components.

# Sub-operations

> Notation note: I use the term "clock cycle" in a way that is analogous to "row in the execution trace".

We wish to define a set of sub-operations our VM can execute. Multiple sub-operations can be executed per clock cycle. Each instruction in the instruction set exposed by the VM is composed of 1 or more sub-operations.

The intention is for sub-operations to be implementable as independent VM circuit relations.

# Control flow

![](/img/protocol-specs/public-vm/avm-control-flow.png)

> Notation note: whenever the VM "sends a signal" to one or more VM components, this is analogous to defining a boolean column in the execution trace that toggles on/off specific functionality

- The instruction controller uses a program counter to read the current opcode to be executed, and send signals to downstream components to execute the opcode
- The state controller reads data from memory into the intermediate registers, using the opcode parameter values
- The VM "algebraic logic unit" executes the opcode given the intermediate register values, and writes output into the intermediate registers
- The state controller writes the output from an intermediate register into memory

## Chiplets

This borrows the chiplet nomenclature from the Miden VM - these components encapsulate functionality that can be effectively defined via an independent sub-circuit (analog in real world = specific part of a CPU chip die)

Chiplets can be developed iteratively i.e. first draft of the AVM only needs a barebones algebraic logic unit.

> We want apply the following design heuristic to chiplets: they are _loosely coupled_ to other AVM components. It should be possible to remove a chiplet and the AVM opcodes it implements, without requiring any upstream changes to the AVM architecture/implementation
