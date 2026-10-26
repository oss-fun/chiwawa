# Component Model

This document describes how chiwawa parses and instantiates components, and what the runtime leaves out.

## Scope

The parser decodes every section of the binary format for the synchronous subset of the proposal.
The runtime instantiates a component and calls its exported functions with every value type of the synchronous subset:
the integers from `s8` to `u64`, `f32`, `f64`, `bool`, `char`, `flags`, UTF-8 strings, lists, records, tuples, enums, option, result and variants.

Left out, and reported as an unsupported feature:
string encodings other than UTF-8, resources, async functions and types, streams and futures, fixed-length lists, map types, and exception tags.
A core module with more than one memory is also rejected at instantiation, since loads and stores address memory 0 only.

## Parsing

`parser::component` reads a component with a stack of builders, one per component or nested core module.

A component refers to its items by number, and it keeps one numbered list per kind of item: core modules, core instances, core functions, core tables, core memories, core globals, core types, components, instances, functions, values and types.
The specification calls each list an index space.
Every definition, import, export and alias appends an entry to one of them, and later definitions name the item by its position in that list.
`Component` in `structure/component.rs` holds the twelve lists as vectors.
Every alias is resolved while reading, so an entry records where its item came from and nothing refers outside its own component.
Types are resolved into trees that carry no indices.

An outer alias of a module or component keeps the entry's type and records the alias itself, as `Outer { count, index }`.
It cannot be copied, because the outer entry may be an import or an instance export, which differ between instantiations of the outer component.

## Instantiation

`ComponentInst::new` turns the parsed description into live objects: a `ModuleInst` per core instance, a `FuncAddr` per core function, a `ComponentFunc` per component function, and so on.
The work is done by a `Linker`.
It has one table per index space, the lists described under Parsing.
A table has one slot per entry of its list.

### Resolving entries

An entry refers to entries in other index spaces, and the binary's order across spaces is not kept per space.
A `canon lower` entry in the core function space refers to the component function space, which refers back to the core function space, which refers to the core instance space.
So the linker cannot fill one space after another; it has to follow the references.

An entry must become exactly one object.
A core instance is asked for by the definition log and again by every alias of its exports.
If each request built a new `ModuleInst`, the aliases would point at a second copy of the module with a memory of its own, and a store through one copy would be invisible through the other.

The linker therefore has one resolver per index space, such as `core_instance(i)` or `func(i)`, and a resolver builds its entry by calling the resolvers of what the entry refers to.
That recursion is the dependency order: nothing is scheduled, and an entry exists before anything that uses it because asking for it is how it gets built.
Every resolver goes through `memo`, which checks the entry's slot first.
The first request runs the resolver's body and stores the result in the slot; every later request returns the stored object.
The tables sit behind `RefCell` so the resolvers can call each other through `&self`.

In the `dbl` example below, forcing core instance 2 asks for core module 1 and for core instance 1.
Core instance 1 is an export bundle naming core function 1, so it asks for core function 1, a `canon lower`, which asks for component function 0, a `canon lift`, which asks for core function 0, an alias of core instance 0's export.
Core instance 0 was built when the log forced it, so that last request is answered from its slot, and the chain unwinds, filling one slot per level.

```wat
(component
  (core module $M (func (export "dbl") (param i32) (result i32) (i32.add (local.get 0) (local.get 0))))
  (core instance $m (instantiate $M))                              ;; core instance 0
  (func $dbl (param "x" u8) (result u8) (canon lift (core func $m "dbl")))   ;; core func 0 is the alias, func 0 the lift
  (core func $dbl' (canon lower (func $dbl)))                      ;; core func 1
  (core module $N
    (import "" "dbl" (func (param i32) (result i32)))
    (func (export "run") (result i32) (call 0 (i32.const 511))))
  (core instance $n (instantiate $N (with "" (instance (export "dbl" (func $dbl'))))))  ;; core instances 1 and 2
  (func (export "run") (result u32) (canon lift (core func $n "run"))))
```

### Order of evaluation

Following references decides only that a dependency comes first.
It says nothing about the order of two entries that do not depend on each other, and three kinds of entry have effects that guest code can observe:
instantiating a core module writes its data and element segments and runs its start function,
instantiating a nested component does the same for every core instance inside it,
and the component's start function runs guest code and may produce values.
The specification evaluates definitions in the order they appear in the binary, so two start functions writing to a shared memory must run in that order.

The parser therefore records those three kinds of definition in `Component::definitions`, in the order read, and `Linker::instantiate` forces them in that order.
Everything else is built when a forced entry, or the export list at the end, asks for it.

### Outer aliases

A component value is a closure: the definition plus one scope per enclosing component.
A scope holds the modules and components the definition's subtree takes from that enclosing component through outer aliases.
The scope is captured when the enclosing component resolves the definition, so an exported component instantiated elsewhere still sees the modules of the instantiation that defined it.

```wat
(component
  (component $C
    (import "m" (core module $M (export "get" (func (result i32)))))
    (component $Inner
      (alias outer 1 (core module 0) (core module $M'))   ;; $C's module 0: the import
      (core instance $m (instantiate $M'))
      (func (export "get") (result u32) (canon lift (core func $m "get"))))
    (export "inner" (component $Inner)))
  (core module $Other (func (export "get") (result i32) (i32.const 1)))
  (core module $M (func (export "get") (result i32) (i32.const 2)))
  (instance $c (instantiate $C (with "m" (core module $M))))
  (alias export $c "inner" (component $I))
  (instance $i (instantiate $I))
  (func (export "get") (alias export $i "get")))
```

`get` must return 2.
When `$c` is instantiated, its linker resolves the definition `$Inner` and captures a scope with `core module 0 = $M`.
The export `inner` is that closure.
`$I` is instantiated at the top level, where core module 0 is `$Other`, but its linker answers the outer alias from the captured scope and instantiates `$M`.
Copying `$C`'s entry into `$Inner` at parse time could not do this, because the entry says "import 0", which means nothing outside `$C`.

### Core instances

A core instance is either an instantiated module or a bundle of exports with no module behind it.
An instantiated module gets its imports from the core instances named by the instantiation arguments.

## Calls across the canonical ABI

A lifted function lowers its component arguments to core values, runs the core function on a runtime of its own, lifts the result, and then calls the `post-return` function if there is one.

`canon lower` produces a host function.
When core code calls it, the host function lifts the core arguments, calls the lifted function, and lowers the result back.
A call from one component instance into another therefore runs on a nested runtime, which is how a trap in the callee reaches the caller.

Lifting follows the canonical ABI for scalars:
a narrow integer is truncated to its width and sign-extended if signed, `bool` is true for any nonzero value, a `char` outside the Unicode scalar values traps, undefined flag bits are dropped, and a NaN becomes the canonical NaN.

### Values in memory

`execution/canon.rs` implements the canonical ABI.
A string or a list is passed as a pointer and a length into the linear memory named by the `memory` option.
A record, tuple or variant is passed in registers when it is small, and in memory otherwise.
Values go to memory when a function has more than 16 flat parameters or more than 1 flat result; the parameters then arrive as one pointer to a tuple, and a lifted function returns a pointer while a lowered function receives one as its last parameter.
Layout follows the spec: each type has an alignment and an element size, a variant stores its case index in the smallest integer that holds the case count, and a flattened variant joins the core types of its payloads.

Lowering a string or list into memory calls the instance's `realloc`.
The runtime clears the instance's `may_leave` flag for the duration of that call and of the `post-return` call.
While the flag is clear, a `canon lower`ed import called from that instance traps with "cannot leave component instance".
A trap in a lifted or lowered call restores the flag, so a failed call does not leave the instance unable to call out.

Every lift and lower carries a `CanonContext`: the resolved `memory`, `realloc` and string encoding of its own definition, and the `may_leave` flag of its own instance.
A lifted function uses the exporting instance's context for its arguments and results, and a lowered function uses the importing instance's, so a value crossing between two instances is written once in each memory.
