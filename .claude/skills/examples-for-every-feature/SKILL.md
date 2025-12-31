---
name: examples-for-every-feature
description: Add minimal, compiling examples for each major new capability: AtomVM boot, module load, NIF call shape, port messaging, tagged ADT exchange, resources.
---

# Examples For Every Feature

## Goals
- Provide "copy-paste runnable" examples
- Make APIs obvious without reading internals

## Where examples go
- Add examples under a new `examples/` folder at repo root, or `src/atomvm_support/examples/` if you must keep it internal.
- Each example should be minimal and compile.

## Required examples for major categories
- VM boot + shutdown
- Load a module from bytes
- Term roundtrip encode/decode
- Tagged ADT serialize/deserialize
- Port send/receive (or simulated harness)
- Resource creation and cleanup

## Style rules
- No hidden magic: show explicit steps
- Add a short comment header at the top explaining what it demonstrates
- Avoid unnecessary dependencies

## Output contract
Any new major feature should ship with:
- at least one example file
- and a short README snippet describing how to run it
