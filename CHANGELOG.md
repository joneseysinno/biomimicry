# Changelog

## 0.2.0 — unreleased

- Rename DNA-layer types to biological vocabulary: `Hyperedge`→`Cistron`,
  `HyperedgeKind`→`CistronKind`, `SpatialHypergraph`/`Hypergraph`→`Grn`;
  Store methods `*_hyperedge`→`*_cistron`, `*_hypergraph`→`*_grn`;
  error `MalformedHyperedge`→`MalformedCistron`. Separates engine vocabulary
  from infinite-db's. No behaviour, hash, or on-disk-format change. Breaking API.

## 0.1.0

- Gardenable M0–M10 scaffold: settle through wall-move, inspector, benches, props.
