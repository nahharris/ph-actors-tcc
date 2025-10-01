## Core Definition and Purpose

The **Actor Model** is a mathematical theory treating "Actors" as universal primitives of digital computation for building scalable, robust information systems. The fundamental hypothesis states: *All physically possible computation can be directly implemented using Actors*.

## Key Concepts and Definitions

### What is an Actor?

When an Actor receives a message, it can **concurrently**:
1. Send messages to (unforgeable) addresses of Actors it has
2. Create new Actors
3. Designate how to handle the next message it receives

### Message Passing Foundation

- **Messages** are the unit of communication
- **Types** enable secure communication with any Actor
- Message passing is **asynchronous** - the sender is decoupled from communications
- An Actor can only communicate with another Actor to which it has an **address**

### Fundamental Principles

**Locality and Security**: In processing a message, an Actor can send messages only to addresses it:
1. Receives in the message
2. Already had before receiving the message
3. Creates while processing the message

**Indeterminacy**: The Actor Model supports indeterminacy because message reception order can affect future behavior, yet the system guarantees service.

## Historical Context and Development

### Origins (1973)
- Created by Carl Hewitt, Peter Bishop, and Richard Steiger
- Inspired by physical laws (unlike previous algebraic models)
- Influenced by: Lisp, Simula-67, Smalltalk-72, Petri Nets, capability systems, and packet switching
- Developed during advent of massive concurrency through client-cloud computing and many-core architectures

### Key Differentiators from Prior Models

**vs. Lambda Calculus**:
- Lambda calculus uses variable substitution (unsuitable for concurrency/shared resources)
- Actor Model can express everything in lambda calculus, but lambda calculus **cannot** express all Actor computations
- Some Actor computations are **exponentially faster** than lambda calculus implementations

**vs. Turing Machines**:
- Turing's model: computation at single location, well-defined sequential states
- Actor Model: computation distributed in space, asynchronous communication, no global well-defined state
- Actor Model can implement **unbounded nondeterminism** (impossible for nondeterministic Turing Machines)

**vs. Process Calculi (π-calculus)**:
- Actor Model: founded on physics
- Process calculi: founded on algebra
- Actor semantics: based on message orderings (Computational Representation Theorem)
- π-calculus semantics: based on structural congruence and bi-simulations

## Major Theoretical Results

### Computational Representation Theorem

For closed systems, the denotation represents all possible behaviors as:

**Denote_S = limit(i→∞) Progression_S^i**

where Progression takes partial behaviors to their next stage.

**Consequence**: There are **uncountably many different Actors**.

**Example**: `Real∎[]` can output any real number between 0 and 1:
```
Real∎[] ≡ [(0 either 1), ⩛Postpone Real∎[]]
```

### Unbounded Nondeterminism

The Actor Model can implement unbounded nondeterminism, proving:

**Theorem**: There are nondeterministic computable functions on integers that cannot be implemented by a nondeterministic Turing machine.

**Proof Example**: The `Unbounded` Actor system can return integers of unbounded size, with graph {start[]⇝0, start[]⇝1, start[]⇝2, …}

This has profound implications:
- **Computation in general cannot be subsumed by logical deduction**
- Mathematical logic alone cannot implement concurrent computation in open systems
- Contradicts Kowalski's claim that "computation could be subsumed by deduction"

## Information System Principles

The Actor Model supports six key principles for information integration:

1. **Persistence**: Information is collected and indexed; no original information is lost
2. **Concurrency**: Work proceeds interactively and concurrently, overlapping in time
3. **Quasi-commutativity**: Information can be used regardless of initiation order
4. **Sponsorship**: Sponsors provide resources (processing, storage, communications)
5. **Pluralism**: Information is heterogeneous, overlapping, often inconsistent; no central arbiter of truth
6. **Provenance**: Information provenance is carefully tracked and recorded

## Inconsistency Robustness

A paradigm shift from **inconsistency denial** and **elimination** to **inconsistency robustness**:

- Information system performance in the face of continual pervasive inconsistencies
- An inference system is **inconsistent** when both a proposition and its negation can be derived
- A **contradiction** is manifest when both a proposition and its negation are asserted (even by different parties)

## Implementation Concepts

### Actor Implementations Example

```actorscript
Actor Account⟨aCurrency⊑Currency⟩[startingBalance:aCurrency]
    currentBalance ≔ startingBalance
    getBalance[]:aCurrency → currentBalance
    deposit[anAmount:aCurrency]:Void → 
        Void afterward currentBalance ≔ currentBalance+anAmount
    withdraw[anAmount:aCurrency]:Void → 
        anAmount>currentBalance � 
            True ⦂ Throw OverdrawnException[]
            False ⦂ Void afterward currentBalance ≔ currentBalance-anAmount
```

### Swiss Cheese Pattern

A programming construct for scheduling concurrent access to shared resources with:
- **Generality**: Ability to program any scheduling policy
- **Performance**: Maximum implementation performance
- **Understandability**: Invariants hold at all observable execution points

Key rule: At most one activity executes "in the cheese" at a time, but the cheese has "holes" where concurrent operations can occur.

### Futures

Futures enable parallel execution and can be chained:

```actorscript
Size∎[aFutureList:Future⟨List⟨String⟩⟩]:Future⟨Integer⟩ ≡
    aFutureList �
        Future List⟨String⟩[] ⦂ Future 0,
        Future List⟨String⟩[aFirst:String, ⩛aRest:Future⟨List⟨String⟩⟩] ⦂ 
            Future aFirst∎length[] + Size∎[aRest]
```

## Organizational Programming (iOrgs)

Based on **authority** and **accountability**:

- Uses hierarchical structure (like sales, accounting, engineering departments)
- Authority is delegated down organizational structures
- Issues are escalated upward
- Authority requires accountability (record keeping, periodic reports)
- Structured around **organizational commitment** (information pledged)

## Practical Implementations

### Actor Systems Evolution

1. **Early Languages**: Act1, Act2, Ani, Ether, Cantor
2. **Erlang Actors**: Process-based, no shared memory, mailbox-based message retrieval
3. **Orleans Actors**: Distributed implementation, transparent message routing across computers, single-threaded message processing
4. **JavaScript Actors**: Browser-based, worker-based parallelism, promise-based futures

### Internet of Things (IoT)

Actors provide standardization for IoT through interface descriptions:

```xml
<Interface name="Account">
    <parameters>
        <type subtypeOf="Currency">aCurrency</type>
    </parameters>
    <handler name="getBalance">
        <arguments/>
        <returns>aCurrency</returns>
    </handler>
    <!-- additional handlers -->
</Interface>
```

## Key Controversies and Resolutions

### The Unbounded Nondeterminism Controversy

**Dijkstra's Position**: Unbounded nondeterminism is impossible to implement (based on "weakest preconditions" for global states)

**Actor Model Resolution**: 
- Unbounded nondeterminism is implementable and necessary
- Required to prove servers provide service to all clients
- Example: readers/writers problem requires unbounded nondeterminism for fairness guarantees

**CSP Evolution**: Initially specified bounded nondeterminism; later versions switched to unbounded nondeterminism to enable service guarantees

### Scheme/Lambda Calculus Debate

**Sussman and Steele (1975) Claim**: "Actors and lambda expressions were identical in implementation"

**Actual Reality**:
- Lambda calculus can express only some sequential/parallel control structures
- Cannot express general Actor concurrency
- Futures cannot be reduced to continuation-passing style
- Actor Model is exponentially faster for many practical applications
- Customers cannot be expressed as lambda expressions (would violate single-response requirement)

## Differences from Classical Object Models

1. **Foundation**: Classical objects based on physical world simulation; Actors based on physics of computation
2. **Type System**: Classical objects are instances of classes in hierarchy; Actors can implement multiple interfaces
3. **Communication**: Classical objects use virtual procedures; Actors use messages

## Performance and Scalability Insights

### Advantages

- Message passing has same overhead as looping and procedure calling
- Primitive Actors can be implemented directly in hardware
- No required overhead for threads, mailboxes, queues, channels, etc.
- Dynamic adjustment for system load and capacity
- Locality (not bound by sequential global memory model)
- Inherent concurrency (not bound by communicating sequential processes)

### Architectural Support

**Cosmic Cube** and **J-Machine** provided hardware support:
- Asynchronous messaging
- Uniform address space (local/nonlocal)
- Actor pipelining capabilities

## Prematurity Question

**Analysis**: Was the Actor Model premature (ahead of its time)?

**Historical Context**:
- For 30+ years after publication (1973), architectures focused on single-thread speed
- For 25+ years, no standard for high-level data structure communication across organizations
- Circumstances have changed: many-core architectures now essential, better communication standards exist

**Conclusion**: By Gerson's criteria, the Actor Model might be considered "before its time" rather than premature:
- Step 1: Did not initially connect to conventional knowledge
- Step 2: Being "rediscovered" in changed context enabling connection

## Current Status and Future

### Industry Adoption

- Erlang: Used in 3G mobile networks worldwide (Ericsson)
- Orleans: Used in high-performance applications (Halo multiplayer games)
- JavaScript: Moving toward Actor-based concurrency with promises/async-await
- C#, Java, JavaScript, Objective C, SystemVerilog: All moving toward Actor Model

### Remaining Challenges

1. Garbage collection for distributed Actors
2. Efficient implementation without exposing low-level details
3. Standardization across platforms
4. Education and adoption barriers

## Key Takeaways

1. **Fundamental Paradigm**: Actors represent a fundamental shift from sequential to inherently concurrent computation models

2. **Mathematical Completeness**: Actor Model is mathematically complete for concurrent computation (more expressive than lambda calculus or Turing machines for concurrent systems)

3. **Practical Scalability**: Designed for massive concurrency (client-cloud computing, many-core architectures)

4. **Inconsistency Management**: Provides foundation for reasoning about inconsistent, distributed information

5. **Physical Grounding**: Unlike algebraic models, grounded in physical laws of computation

6. **Organizational Metaphor**: Provides intuitive framework through organizational programming (iOrgs)

7. **Ongoing Evolution**: Continues to influence modern programming languages and distributed systems design

The Actor Model represents a complete rethinking of concurrent computation, moving from global state machines to distributed, asynchronous message-passing systems that better reflect the physical reality of modern computing systems.