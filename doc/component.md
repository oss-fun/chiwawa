# Component Model

This document is about how Chiwawa parses and instantiates components, and about the functions that the runtime does not implement.

## Scope

The parser decodes each section of the binary format for the synchronous subset of the proposal.
The runtime instantiates a component and calls its exported functions.
The calls can use each value type of the synchronous subset:

- the integers from `s8` to `u64`
- `f32` and `f64`
- `bool`, `char`, and `flags`
- strings
- lists, records, tuples, and enums
- option, result, and variants
- `own` and `borrow` handles of resources

The runtime does not implement these items, and it reports each of them as an unsupported feature:

- async functions and types
- streams and futures
- fixed-length lists
- map types
- exception tags

Instantiation also rejects a core module with more than one memory, because loads and stores address only memory 0.

## Parser

`parser::component` reads a component with a stack of builders, one for each component or nested core module.

A component refers to its items by number, and it keeps one numbered list for each category of item:

- core modules, core instances, core functions, core tables, core memories, core globals, and core types
- components, instances, functions, values, and types

The specification calls each list an index space.
Each definition, import, export, and alias adds an entry to one of the lists.
Subsequent definitions refer to the item by its position in that list.
`Component` in `structure/component.rs` contains the twelve lists as vectors.

The parser resolves each alias while it reads.
Thus an entry records the source of its item, and no entry refers to an item outside its component.
The parser resolves types into trees that contain no indices.

An outer alias of a module or component keeps the type of the entry and records the alias, as `Outer { count, index }`.
The parser cannot copy the entry, because the outer entry can be an import or an instance export.
Imports and instance exports are different between instantiations of the outer component.

## Instantiation

`ComponentInst::new` changes the parsed component into live objects:

- a `ModuleInst` for each core instance
- a `FuncAddr` for each core function
- a `ComponentFunc` for each component function
- an equivalent object for each of the other categories of item

A `Linker` does the work.
It has one table for each index space (see the lists in the Parser section).
A table has one slot for each entry of its list.

### Resolution of entries

An entry refers to entries in other index spaces.
The binary puts the entries of all spaces in one sequence, but each list keeps only the sequence of the entries in that list.
A `canon lower` entry in the core function space refers to the component function space.
That space refers back to the core function space, which refers to the core instance space.
Thus the linker cannot fill one space and then the next, and it must follow the references.

An entry must become one object and only one.
The definition log uses a core instance, and each alias of the exports of that instance uses it again.
A new `ModuleInst` for each of these operations is incorrect.
Then the aliases point at a second copy of the module.
That copy has a different memory, and a store through one copy does not change the other.

Thus the linker has one resolver for each index space, for example `core_instance(i)` or `func(i)`.
A resolver builds its entry with calls to the resolvers of the entries that its entry refers to.
That recursion is the dependency sequence, and the linker does not schedule the entries.
A call to a resolver builds the entry, and thus the linker builds an entry before the entries that use it.

Each resolver goes through `memo`, which examines the slot of the entry first.
The first call runs the body of the resolver and stores the result in the slot.
Each subsequent call returns the stored object.
The tables are in a `RefCell`, and thus the resolvers can call each other through `&self`.

The `dbl` example after this list shows the resolver calls that occur when the linker forces core instance 2:

1. The resolver of core instance 2 calls the resolvers of core module 1 and core instance 1.
2. Core instance 1 is a bundle of exports that names core function 1, and thus its resolver calls the resolver of core function 1.
3. Core function 1 is a `canon lower`, and its resolver calls the resolver of component function 0.
4. Component function 0 is a `canon lift`, and its resolver calls the resolver of core function 0.
5. Core function 0 is an alias of an export of core instance 0, and its resolver calls the resolver of core instance 0.
6. The log forced core instance 0 before this point, and thus that last call returns the object from the slot.
7. The calls then return in the opposite sequence, and each level fills one slot.

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
### Sequence of evaluation

The references set only that a dependency comes first.
They do not set the sequence of two entries that have no dependency between them.
Three categories of entry have effects that guest code can see:

- The instantiation of a core module writes its data and element segments and runs its start function.
- The instantiation of a nested component does the same for each core instance in it.
- The start function of the component runs guest code and can make values.

The specification evaluates definitions in their sequence in the binary.
Thus two start functions that write to a shared memory must run in that sequence.

Thus the parser records those three categories of definition in `Component::definitions`, in the sequence in which it reads them.
`Linker::instantiate` forces them in that sequence.
The linker builds all other entries when a forced entry, or the export list at the end, uses them.

### Outer aliases

A component value is a closure.
It contains the definition and one scope for each component that contains the definition.
A scope contains the modules and components that the subtree of the definition gets from that outer component through outer aliases.
The linker captures the scope when the outer component resolves the definition.
Thus an exported component continues to use the modules of the instantiation that defined it, also when a different location instantiates it.

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
During the instantiation of `$c`, its linker resolves the definition `$Inner` and captures a scope with `core module 0 = $M`.
The export `inner` is that closure.

The instantiation of `$I` occurs at the top level, where core module 0 is `$Other`.
But its linker resolves the outer alias from the captured scope and instantiates `$M`.
A copy of the entry of `$C` into `$Inner` at parse time could not do this.
The entry contains "import 0", which refers to no item outside `$C`.

### Core instances

A core instance is an instantiated module or a bundle of exports without a module.
An instantiated module gets its imports from the core instances that the instantiation arguments name.

## Calls across the canonical ABI

`canon lift` makes a component function from a core function.
Other components can use the component function.
`canon lower` makes a core function from a component function.
The core code of the same component can import and call the core function.

A lifted function does these steps:

1. It lowers its component arguments to core values.
2. It runs the core function on a different runtime.
3. It lifts the result.
4. It calls the `post-return` function, if there is one.

`canon lower` makes a host function.
When core code calls it, the host function lifts the core arguments, calls the lifted function, and lowers the result back.
Thus a call from one component instance into a different instance runs on a nested runtime.
This is how a trap in the callee goes to the caller.

A lift follows the canonical ABI for scalars:

- It truncates a narrow integer to its width, and it sign-extends the integer if the type is signed.
- `bool` is true for each nonzero value.
- A `char` outside the Unicode scalar values traps.
- It drops undefined flag bits.
- A NaN becomes the canonical NaN.

### Values in memory

`execution/canon.rs` implements the canonical ABI.
A call gives a string or a list as a pointer and a length into the linear memory that the `memory` option names.
A call gives a record, tuple, or variant in registers when it is small, and in memory when it is not.
Values go to memory when a function has more than 16 flat parameters or more than 1 flat result.
The function then receives the parameters as one pointer to a tuple.
A lifted function returns a pointer, and a lowered function receives a pointer as its last parameter.

The layout follows the specification:

- Each type has an alignment and an element size.
- A variant stores its case index in the smallest integer that can contain the case count.
- A flattened variant joins the core types of its payloads.

To lower a string or a list into memory, the runtime calls the `realloc` of the instance.
The runtime clears the `may_leave` flag of the instance during that call and during the `post-return` call.
While the flag is clear, an import that `canon lower` made traps and reports `cannot leave component instance` when that instance calls it.
A trap in a lifted or lowered call restores the flag.
Thus a failed call does not cause a state in which the instance cannot call out.

The `string-encoding` option of a lift or a lower selects UTF-8, UTF-16, or Latin-1 plus UTF-16.
A lift reads a string in the encoding of that lift.
A lower writes a string in the encoding of that lower.
Thus the runtime changes the encoding of a string that goes between two instances with different encodings.

In Latin-1 plus UTF-16, the high bit of the length marks UTF-16 data.
The runtime writes Latin-1 when each character fits in one byte, and UTF-16 when a character does not.
A string with bytes that the encoding does not permit causes a trap.

The specification records the source encoding of each string and uses it to select the memory sizes.
chiwawa does not record the source encoding.
It uses the memory sizes that the specification gives for a UTF-8 source.
The bytes in memory are the same.

Each lift and each lower has a `CanonContext`.
It contains the resolved `memory`, `realloc`, and string encoding of its definition, and the `may_leave` flag of its instance.
A lifted function uses the context of the instance that exports it for its arguments and results.
A lowered function uses the context of the instance that imports it.
Thus the runtime writes a value that goes between two instances one time in each memory.

### Resources

A resource is a type that one component instance implements.
Other instances hold handles to a resource, and they do not hold the value.
A component instance that has a resource type definition makes a new resource type.
Thus two instances of one component have two different resource types.

A component can import a resource type, or receive one from an instance export.
At instantiation, the runtime connects that type to the resource type of the instance that exports it.
The runtime keeps the connection in `ExternVal::Type`.

Each component instance has a handle table.
The table holds one `ResourceHandle` for each index.
Core code refers to a handle by its index, as an `i32`, and the runtime reads the handle.
The table is in `InstanceState`, together with the `may_leave` flag.

The runtime does not use index 0.
The runtime uses a removed slot again before the table becomes larger.
Of the removed slots, it uses the most recent one first.
A handle records its resource type, the representation, and if it is an `own` or a `borrow`.

The linker makes a core function for a `canon resource.new`, a `canon resource.drop`, or a `canon resource.rep`.
`resource.new` adds an `own` handle for a representation.
`resource.rep` reads the representation behind a handle.
`resource.drop` removes a handle.
When the handle is an `own`, `resource.drop` also calls the destructor of the resource with the representation.

A lift of an `own` removes the handle from the table of the instance and gives the representation to the callee.
A lower of an `own` adds a handle to the table of the instance that receives it.
A lift of a `borrow` reads the representation and increases the borrow count of the handle until the call returns.
A lower of a `borrow` adds a `borrow` handle.
When the instance that receives the `borrow` implements the resource, the lower adds no handle.
It gives the representation.

Two conditions cause a trap:

- The callee did not call `resource.drop` for each `borrow` handle that it received before it returned.
- A lift or `resource.drop` tries to remove an `own` handle while its borrow count is not zero.

The call keeps these counts in a `CallScope`.
