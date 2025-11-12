# The Dynamic Enum Pattern: Type-Safe Runtime Selection in Rust

The pattern in this document is a direct, practical application of the ideas described in the Rust community as "enum-based polymorphism," extended with real-world cryptographic and protocol negotiation use cases. For more background, see the [Rust Patterns: Enum-based Polymorphism](https://rust-unofficial.github.io/patterns/patterns/behavioural/enum-polymorphism.html) and [Rust Book: Enums](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html).

## Overview

The "Dynamic Enum Pattern" is a Rust design approach for type-safe, zero-cost runtime selection among a small, fixed set of alternatives. It is especially useful in embedded, cryptographic, and protocol code where you need to choose between several statically-typed implementations at runtime, but want to avoid heap allocation, dynamic dispatch, or loss of static guarantees.

## Motivation

Many systems require runtime selection of algorithms or behaviors (e.g., cryptographic hash functions, protocol variants, hardware drivers). In C, this is often done with function pointers or unions. In Rust, we want:
- **Type safety**: Only valid choices are possible.
- **Zero-cost abstraction**: No heap allocation or virtual dispatch.
- **Extensibility**: Easy to add new alternatives.
- **Ergonomics**: Simple, readable code.

## The Pattern

### 1. Define a Trait for the Common API

```rust
pub trait Operation {
    fn do_work(&mut self, input: &[u8]) -> Result<(), Error>;
    fn finish(self) -> Result<Output, Error>;
}
```

### 2. Implement the Trait for Each Concrete Type

```rust
pub struct ImplA { /* ... */ }
pub struct ImplB { /* ... */ }

impl Operation for ImplA { /* ... */ }
impl Operation for ImplB { /* ... */ }
```

### 3. Define an Enum Wrapping Each Type

```rust
pub enum DynamicOperation {
    A(ImplA),
    B(ImplB),
}
```

### 4. Implement the Trait for the Enum by Delegation

```rust
impl Operation for DynamicOperation {
    fn do_work(&mut self, input: &[u8]) -> Result<(), Error> {
        match self {
            DynamicOperation::A(a) => a.do_work(input),
            DynamicOperation::B(b) => b.do_work(input),
        }
    }
    fn finish(self) -> Result<Output, Error> {
        match self {
            DynamicOperation::A(a) => a.finish(),
            DynamicOperation::B(b) => b.finish(),
        }
    }
}
```

### 5. Construct the Enum at Runtime

```rust
fn select_operation(choice: Choice) -> DynamicOperation {
    match choice {
        Choice::A => DynamicOperation::A(ImplA::new()),
        Choice::B => DynamicOperation::B(ImplB::new()),
    }
}
```

---

**Try it yourself:** [Rust Playground Example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2024&gist=bec7a944ae42b81cfa986ff795848ce2)

---

## Advantages

- **Type safety**: Only valid variants can be constructed.
- **No heap allocation**: The enum is stack-allocated.
- **No dynamic dispatch**: The compiler generates efficient code for each case.
- **Extensible**: Add new variants as needed.
- **Pattern matching**: You can match on the enum for custom logic.

## When to Use

- The set of alternatives is small and known at compile time.
- You want to avoid heap allocation and dynamic dispatch.
- You need runtime selection but want to keep static guarantees.

## When Not to Use

- The set of alternatives is large or open-ended.
- You need to store many different types in a collection (consider trait objects or enums with boxed data).
- You require plugin-like extensibility at runtime.

## Real-World Examples

- Cryptographic algorithm selection (e.g., HMAC with SHA-256, SHA-384, SHA-512)
- Protocol negotiation (e.g., SPDM, TLS cipher suites)
- Hardware driver selection (e.g., different peripherals)

## Comparison: Enum vs Trait Object

| Aspect           | Enum Pattern         | Trait Object (`Box<dyn Trait>`) |
|------------------|---------------------|---------------------------------|
| Heap allocation  | No                  | Usually yes                     |
| Dispatch         | Static (match)      | Dynamic (vtable)                |
| Type safety      | Strong (fixed set)  | Weaker (any implementor)        |
| Extensibility    | Add variants        | Any type implementing trait     |
| Performance      | Best                | Slight overhead                 |

## Example: DynamicHasher

See `DYNAMIC_HASHER_FOR_NEW_RUSTACEANS.md` for a concrete example applying this pattern to cryptographic hashers.

## Case Study: DynamicHasher and Digest Traits

Let's apply the Dynamic Enum Pattern to a real-world scenario: cryptographic digest (hash) algorithm selection in an embedded or protocol context.

### Problem

Suppose you are implementing a protocol (like SPDM) that negotiates which HMAC algorithm to use. Your hardware supports several SHA-2 variants (SHA-256, SHA-384, SHA-512). You want to:
- Select the algorithm at runtime (after negotiation)
- Use Rust's trait system for code reuse and safety
- Avoid heap allocation and dynamic dispatch

### Step 1: Define Digest Traits

```rust
pub trait DigestInit {
    fn init(&mut self) -> Result<(), Error>;
}

pub trait DigestOp {
    fn update(&mut self, data: &[u8]) -> Result<(), Error>;
    fn finalize(self) -> Result<DigestResult, Error>;
}
```

### Step 2: Implement Traits for Each Algorithm

```rust
pub struct Hasher<'a, Algo> { /* ... */ }

impl<'a> DigestInit for Hasher<'a, Sha2_256> { /* ... */ }
impl<'a> DigestOp for Hasher<'a, Sha2_256> { /* ... */ }
// ...repeat for Sha2_384, Sha2_512...
```

### Step 3: Define the Dynamic Enum

```rust
pub enum DynamicHasher<'a> {
    Sha256(Hasher<'a, Sha2_256>),
    Sha384(Hasher<'a, Sha2_384>),
    Sha512(Hasher<'a, Sha2_512>),
}
```

### Step 4: Implement Methods by Delegation

```rust
impl<'a> DynamicHasher<'a> {
    pub fn update(&mut self, data: &[u8]) -> Result<(), Error> {
        match self {
            DynamicHasher::Sha256(h) => h.update(data),
            DynamicHasher::Sha384(h) => h.update(data),
            DynamicHasher::Sha512(h) => h.update(data),
        }
    }
    pub fn finalize(self) -> Result<DigestResult, Error> {
        match self {
            DynamicHasher::Sha256(h) => h.finalize(),
            DynamicHasher::Sha384(h) => h.finalize(),
            DynamicHasher::Sha512(h) => h.finalize(),
        }
    }
}
```

### Step 5: Construct at Runtime

```rust
pub fn new_dynamic_hasher<'a>(hmac: &'a mut Hmac, algo: SupportedAlgorithm) -> Result<DynamicHasher<'a>, Error> {
    match algo {
        SupportedAlgorithm::Sha256 => Ok(DynamicHasher::Sha256(hmac.init(Sha2_256)?)),
        SupportedAlgorithm::Sha384 => Ok(DynamicHasher::Sha384(hmac.init(Sha2_384>?)),
        SupportedAlgorithm::Sha512 => Ok(DynamicHasher::Sha512(hmac.init(Sha2_512>?)),
    }
}
```

### Benefits in Practice

- **Type safety**: Only valid hashers can be constructed.
- **No heap allocation**: All data is stack-allocated.
- **No dynamic dispatch**: The compiler generates efficient code for each algorithm.
- **Extensible**: Add new algorithms by adding enum variants and trait impls.

### Why Not Trait Objects?

A trait object (e.g., `Box<dyn DigestOp>`) would require heap allocation or indirection, and would lose the static guarantees about which algorithms are supported. The enum approach is more efficient and robust for this use case.

### Summary

The Dynamic Enum Pattern, as applied to digest traits and the `DynamicHasher`, enables safe, efficient, and ergonomic runtime selection of cryptographic algorithms in Rust. This approach is ideal for embedded and protocol code where performance and correctness are critical.

## Summary

The Dynamic Enum Pattern is a powerful, idiomatic Rust technique for type-safe, efficient runtime selection among a fixed set of alternatives. It leverages enums and traits to provide both static guarantees and runtime flexibility, making it ideal for embedded, cryptographic, and protocol code. In summary, this pattern leverages Rust's enums and traits to provide both static guarantees and runtime flexibility, as described in the [Rust Book: Traits](https://doc.rust-lang.org/book/ch10-02-traits.html) and the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/type-safety.html#c-enum). For embedded use cases, see also the [Rust Embedded Book: Zero-cost Abstractions](https://docs.rust-embedded.org/book/design-patterns/zero-cost-abstractions.html). For a discussion of tradeoffs with trait objects, see [Dynamic Dispatch vs. Enums in Rust (blog)](https://deterministic.space/elegant-apis-in-rust.html).

## References

- [Rust Book: Enums](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html)
- [Rust Book: Traits](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [Rust API Guidelines: Enums](https://rust-lang.github.io/api-guidelines/type-safety.html#c-enum)
- [Rust Embedded Book: Zero-cost Abstractions](https://docs.rust-embedded.org/book/design-patterns/zero-cost-abstractions.html)
- [Rust Patterns: Enum-based Polymorphism](https://rust-unofficial.github.io/patterns/patterns/behavioural/enum-polymorphism.html)
- [Dynamic Dispatch vs. Enums in Rust (blog)](https://deterministic.space/elegant-apis-in-rust.html)
