# Validus Backend Engineer Exercise

## Overview

The aim of this exercise is to develop a trade approval system, enabling users to submit trade details for approval per some established protocol, and for the system to support various workflows given their trade details.

## Prerequisites

- Protobuf compiler `protoc`, developed on my Debian machine with libprotobuf.
- Standard rust tools (`cargo`, `rustc` etc).

## Deliverables Met

I consider this assignment complete in that it has met all its deliverables:

- I've implemented the workflow in Rust, with the API library and example simple gRPC service in seperate subcrates.

- Implemented simple unit tests which can be ran with `cargo test`, and one of the unit tests in `history.rs` will need to be ran with `cargo test -- --test-threads=1 --ignored`

- I've provided code documentation for the API, and both the example gRPC service and library integration test `example.rs` aswell.

## Reflection

As much as time constraints were a major hinderence to scope for this assignment, I think it's important to recognize areas of improvement. Here are some things in retrospect I think I could've done better:

- Started with the gRPC service definintions to layout the problem space instead of the library. It would have lead me down a different path, not following a generic type state pattern but potentially one that would permit dynamic dispatch.

- Could have provided a better mocked security, and treated the "approver" and "requester" more as roles instead of mutually exclusive states (imagine you could have a user that is both a "requester" and "approver").

When considering what went well, here are some areas of the assignment I'm happy with:

- Pursued a type state pattern when designing the library, relying on Rust's compile time checks to ensure state machine consistency.

- A multi-threaded environment was considered for the History API.

- Was able to extend past the requirements by implementing a gRPC server.
