# Proposal 1 - Combine OOP and FP

The object of this work is to design a model for creating applications that combine the strengths of both OOP and FP. The core idea is to pick the best features of each paradigm, combine them in a way to produce good software.
In the context of this work, good software means:

- Readable: the source code is easy to read and understand
- Writable: the source code is easy to be modified and extended
- Testable: the application needs to have a good test coverage
- Performative: the application cannot be slow
- Safe: the application must be resilient to crashes and exploitation
  It's notable that OOP is the industry favorite, while FP has more appeal in the academy. So we want to use OOP as a reference to guide what the industry needs, while applying FP concepts to model it.

## Keywords

- Functional programming
- Object-Oriented programming
- Application design

## Roadmap

- Objectives
- OOP Concepts
- FP Concepts
- Methodology
- Building the new approach
- Study Case
- Conclusions
- Future Works

# Proposal 2 - Refactor Patch-Hub with the Actor Model

`patch-hub` is hitting a limit on features that can be added due to the rigidity
of Rust single-threaded applications. This proposal tries to show an approach using
the Actor Model to refactor the project and analyze the flexibility gains in the app
structure.

## Roadmap

- Objectives
- [Actor Model Concepts](./Actor Model)
- [Patch Hub Current Stage](./Patch Hub)
- Methodology
- Patch Hub w/ Actor Model
- Conclusions (qualitative and quantitative analysis)
- Future Works
