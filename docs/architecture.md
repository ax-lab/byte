Byte Language Architecture
==========================

This document describes the overall architecture of the language compiler.

--------------------------------------------------------------------------------

# Basic definitions

## Span

The compiler uses a global coordinate system to address source code, program
elements, scopes, and other language definitions.

Elements are addressed using a numeric `Index` + `Offset`, usually derived from
the source code location.

A `Span` refers to an entire range of offsets by adding a `Length` element to
the above.

Any source file is mapped to a unique `Index` number with each byte mapped to
its `Offset` within the file. This provides a unique address to each byte in
the program source code, across all files.

Most program elements will inherit the `Span` from the original source code
region that generated them.

Occasionally, non-source addresses may be used for dynamically generated
elements that do not map directly to the source code.

Many core language concepts, such as scopes, are tied to a `Span`.

## Node 

`Nodes` represent any language element, from low-level tokens and raw source
text to high-level semantic elements, such as an entire module or a type.

A node is __immutable__ and defined by its `Value`, `Span`, and unique `ID`:

  - The `Value` relates to the language element being represented by the
    node and provides all necessary information about it.

  - The `Span` will generally be the source code span from which the node
    was parsed or derived. The `Span` __is not unique__ to a node and can
    overlap.

  - The `ID` is an unique incremental numeric value that is associated
    with each node based on the creation order within the program.

## Segments

A `Segment` consists of a `Span` and a list of `Node`:

  - The `Span` for a segment __is globally unique__, with no two segments
    sharing the exact same span.

  - `Spans` for different segments __must not overlap partially__. If two
    segments overlap at all, then one of them must be entirely contained
    within the other.

Nodes in a segment also follow a set of rules:

  - Each `node` must only be part of a single segment. But note that nodes
    may be moved between segments.

  - Any nodes in a segment __must not overlap__ other nodes in that segment.
  
  - The node __must be entirely contained__ within the segment `Span`.

  - Nodes within a segment are __sorted by their offset__.

A segment __span is immutable__. Once created, the segment cannot be modified
or destroyed.

The segment __node list is mutable__. Nodes can be added, removed, and moved
between segments, as long as the node invariants above are maintained.

## Segment / Node relationship

Segments are processed independently from one another.

That said, segment rules naturally organize segments in a hierarchical tree,
where a segment can be contained and contain other segments.

Since nodes can only ever be part of a single segment, they become leafs
of the above segment tree.

This hierarchy, allows to view segments and nodes as a single tree and
implement hierarchical queries based on the above structure:

  - A node has an intrinsic parent in the segment that contains it.

  - A node successor and predecessor can be found by navigating the segment
    tree and finding the first non-overlapping node with a greater or lesser
    offset than the node.

  - A node parent can be found by navigating up the segment tree and finding
    the closest node that contains the node offset.

  - Nodes may also reference segments as part of their value. But note that
    a segment has no concept of a parent node.

Note that by considering only the offset, the above rules allow nodes to
overlap in weird ways between different segments. In practice, this should
not generally be a problem.

Nodes without a segment are also allowed by the rules (e.g. deleted nodes).
Those are also considered by the node processing, but will generally not have
much effect.

## Node Key

The node value also implicitly defines a `node key` which is used to bind
operations to nodes.

The node key is derived entirely from the value and __is not unique__. Nodes
that share the same key will be bound to the same operators.

## Operators and Bindings

Operators are how nodes and segments are processed.

An operator is bound to a particular __node key__ within a __scope__ given by
a `span` value and with a set __precedence__.

Only nodes with the _given key_ and with an _offset within the scope_ are bound
to an operator.

The binding of a `node key` + `scope` is immutable, but an overlapping (but
distinct) span can be rebound to another operator using the same key.

As with segments, partial overlapping of binding scopes is forbidden. When
resolving a binding from overlapping ranges, the most specific range is used.

The above rules guarantee that any coordinate can only ever map to a single
operator for a given key.

--------------------------------------------------------------------------------

# Node resolution

Node resolution is the process of successively applying operators to nodes and
segments until no more operators apply.

Each successive application is a resolution step.

Each node resolution step is _transactional_. This means that any effects from
operators are applied as an __atomic operation__ at the end of the step and
only visible at the next step.

## Scopes

A `Scope` defines an area of effect for an operator. It is defined as a `Span`.

The scope applies to a `Node` if its `Offset` is within the scope `Span`.

## Binding

Operators are bound as a mapping `(Key, Scope) -> (Operator, Precedence)`.

A given `Key` + `Scope` binding is __immutable__, however:

  - New bindings can be defined by operators during node resolution;
  - A new `(Key, Scope)` pair can overlap an existing one, provided:
    - The overlap is not partial, that is, either the new scope is contained
      or it contains the existing scope.
    - The scopes are not equal (bindings are immutable).
    - (note that the above rules are the same as for segments).

Given the above, it follows that, at any given step, the entire program space
is segmented into distinct `Binding` tuples defined by:

    (Key, Span, Precedence, Operator)

It also follows, that all nodes in the program can be uniquely mapped into
the existing `Bindings` given their `Key` and `Offset`.

## Binding Resolution

The bindings and their respective nodes are sorted into a global heap
by their precedence value.

For each resolution step, the next highest precedence binding is extracted
from the heap and its nodes processed.

As new nodes are created, they are sorted into the existing bindings:

  - If a binding was already processed, then it is reinserted into the heap.
  - When processing a binding, nodes are processed only once. In case a binding
    is reprocessed, only new nodes are considered.

## Node Processing

Operators bind to nodes but apply to segments.

At each resolution step, the segments respective to the bound nodes are
collected and input to the respective operators.

Each segment containing a bound node is processed only once.

Operators have full write access to any segment. They may also create nodes
and define new bindings, as well as generate errors.

If two operators have the same precedence, they will not see each other's
modifications. Both will see the same initial state, and their effects will
be applied at the end of the step.

## Conflict Resolution

If more than one operator is applied in a single resolution steps, the operator
effects must be merged.

In the event that operator effects cannot be merged, this generates a conflict
error and the node resolution is aborted.

__Operator order__

Operators must have a well defined order that is consistent across runs.

Relying on the order is undefined behavior, and in general should not even be
possible without triggering a conflict error.

The only effect operator order has is in the order operator changes are applied,
which may affect conflict reporting messages.

The ID of new nodes also follows operator order as a tie-breaker.

__New bindings__

  - Declaring a binding with the same `Key` + `Scope` will generate a conflict.

__New nodes__

  - Creating new nodes cannot generate a conflict.
  - New node ID generation use operator order as a tie breaker.

__New segments__

  - New segments must be subject to the segment invariants.
  - Any overlap between segments created by different operators is a conflict.

__Adding nodes__

  - Adding nodes to a segment is subject to the node invariants.
  - Added nodes can conflict if they overlap.

__Removing nodes__

  - Removing the same node twice won't generate a conflict. But note that
    adding that node back to different segments will.

__Moving nodes__

  - A node cannot be moved to different segments in the same step.
