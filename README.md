# The Noir Programming Language

<div align="center">
  <picture>
    <img src="./noir-logo.png" alt="The Noir Programming Language" width="35%">
  </picture>

[Website][Noir] | [Getting started] | [Documentation] | [Contributing]
</div>

## 🌟 About

Noir is a Domain Specific Language for SNARK proving systems. It has been designed to work with any ACIR-compatible proving system.

### Key Features:

- 🔒 Security and privacy as core priorities
- 🛠 Compatibility with various proving systems
- 📝 Intuitive Rust-like syntax
- 🚀 Optimized performance
- 🔧 Extensible architecture

**Project Status:** Early development (Alpha). Not suitable for production use.

## 🚀 Quick Start

### System Requirements

- Rust version 1.75.0 or higher
- Cargo (Rust package manager)
- Git

### Installation

Detailed installation instructions are available in the [documentation][Getting started].

## 📚 Usage Examples

```nior
fn main(x: Field, y: pub Field) {
    assert(x * x == y);
}
```

More examples can be found in the [Awesome Noir](https://github.com/noir-lang/awesome-noir) repository.

## 🤝 Community and Support

- [Forum][Forum] - Discussions and help
- [Discord][Discord] - Chat with developers
- [Documentation][Documentation] - Comprehensive guides

## 🛠 Development and Contributing

We welcome contributions to the project! Before you start:

1. Read the [contribution guidelines][Contributing]
2. Check open issues
3. Discuss planned changes in Discord or on the forum

### Development Priorities

- Code safety and reliability
- Developer experience improvements
- Feature expansion
- Performance optimization

## 📄 License

Noir is free and open source. It is distributed under a dual license. (MIT/APACHE)

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this 
repository by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any 
additional terms or conditions.

[Noir]: https://www.noir-lang.org/
[Getting Started]: https://noir-lang.org/docs/getting_started/quick_start/
[Forum]: https://forum.aztec.network/c/noir
[Discord]: https://discord.gg/JtqzkdeQ6G
[Documentation]: https://noir-lang.org/docs
[Contributing]: CONTRIBUTING.md
