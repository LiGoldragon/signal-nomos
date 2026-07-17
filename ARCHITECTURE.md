# Architecture
This pure contract crate owns the typed binary public operations for the nomos daemon. Runtime, storage, actors, and policy remain in the daemon repository. Git dependencies are exact pushed revisions.

## Revisable leans
- **Signal-frame bypass.** This contract exposes raw rkyv `encode_request`/`encode_reply` payloads, and its daemon frames them with a hand-rolled u32-length + rkyv envelope rather than the workspace's shared `signal-frame` kernel. That trades away `signal-frame`'s short-header tap-anywhere observability — the uniform exchange framing readable at any hop. The lean holds while the prototype's point-to-point socket suffices. Revise it when cross-hop observability, shared handshake or version negotiation, or a common frame taxonomy is needed, adopting `signal-frame`'s `ExchangeFrame` as the transport.
