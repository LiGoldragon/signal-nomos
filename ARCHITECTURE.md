# Architecture

This crate is the pure typed binary contract for the stateful Nomos daemon. It
owns request, reply, refusal, receipt, selector, and validated archive value
types. It owns no database, recovery, actor, evaluator, fixture, central-storage
client, or authorization implementation.

## Stateful Boundary

`Deploy` carries a component-owned opaque slot, a compare-and-set expectation,
canonical independently stored Nomos Capsule and authenticated projection
archives, and ordered per-slot `GenerationClass` metadata. The slot/CAS/metadata
shapes are `[to-be-reviewed-by-psyche]`; po2.8 owns retiring the temporary
external generation selection.

`Repoint` selects an already seated full Capsule identity or an ephemeral
slot-scoped lowercase hexadecimal short display. Full identities alone are
durable. Rollback is a repoint.

`AdvanceProjection` is separate from deployment. It carries one authenticated
projection bound to an existing Capsule and an expected prior version. When a
translator receipt is unavailable, the daemon admits the operation only from
its configured admin Unix peer `[to-be-reviewed-by-psyche]`; caller bytes alone
never authenticate it.

`Transform` carries a live or seated selector and a direct encoded
Ethos-population/NameTree archive. The response carries the checked native
Logos-population/NameTree archive plus the admission-time Capsule, slot
generation, and projection version.

## Validation

Wire decoding is followed by `Request::validate`. Deploy archive validation
restores the canonical `SealedNomosCapsule`, restores the integrity-protected
NameTree projection, recomputes content identity, and verifies the projection
belongs to that exact Capsule before the daemon may begin a storage
transaction. Stateful checks, Unix-peer authority, concrete Ethos NameTree
validation, and native evaluation remain in `nomos-engine`.

## Transport Lean

The raw rkyv payload remains inside the currently shared frame envelope. The
transport framing choice is independent of the state and identity laws and can
be revised without changing this domain contract.
