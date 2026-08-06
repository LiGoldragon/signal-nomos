# Architecture

`signal-nomos` owns the small wire vocabulary that remains independently
meaningful at a Nomos seating boundary: opaque slot identity, monotone slot
generation, compare-and-set expectation, and full or resolver-relative Capsule
selection.

The crate is a value library. It has no daemon, request/reply operation algebra,
storage lifecycle, sealed population or Capsule representation, projection
archive, generation selection, evaluator, authorization system, or fixture
package. The present authority-sealed bootstrap transformation has no process
boundary, so those concepts have no current home here.

Restored values with private invariants implement `WireInvariant`. A short
Capsule display is explicitly ephemeral and resolver-relative; only a full
Capsule identity is persistent. A full selector validates that its identity is
of the Nomos kind.
