# Changelog

## 0.4.0 — M12 blocks linker

Composition release. Organisms are assemblable from named DNA fragments; the linker runs **above** `compile` and does not change it.

- **`blocks` module:** `Block`, `Manifest` (TOML), `link` → one GRN + `OrganismGenotype` + ganglion templates.
- **Total qualification:** authors write local kinds; linker rewrites every Receptor/Signal role and `TransductionSpec` kind to `block::local`.
- **Bridge cistrons:** wires are synthesised interneurons (`Forward` transduction), not rename-in-place — visible, linkage-independent block identity.
- **Infer-when-unambiguous:** one matching export wires automatically; two or more → `AmbiguousExport`.
- **All-errors-collected validator:** `UnsatisfiedImport`, `ShapeMismatch`, `CyclicRequire`, `VersionConflict`, …
- **`BlockSource`:** `MemoryBlockSource` + `DirBlockSource` (network registry typed-deferred).
- **`TransductionKind::Forward`:** payload-preserving identity for bridges.
- **`SignalKind::qualified`:** now formats `block::local` (M11 stub activated).
- Versions: exact pins in the manifest; ranges checked; no solver (`VersionSolveUnavailable`).

## 0.3.0 — M11 value + effector

Breaking genotype / semantics release. **M7 snapshots taken under `0.2.x` are not replayable under `0.3.0`.**

- **GeneId break:** `Cistron` now carries an optional `TransductionSpec`; the spec is part of the canonical form hashed into `GeneId`. Identical topology with different enzymes are different genes.
- **Cascade pipeline semantics:** `Cascade::run` chains step outputs into the next step (pipeline). Pre-0.3.0 fan-out (every step saw the original input; outputs concatenated) is available explicitly as `TransductionKind::Fanout`.
- **Value lattice:** typed `Value` is the payload authority; `body` is its canonical encoding.
- **Effectors:** Phase 2 writes leave the signal stream via `EffectorSink` / `MemoryEffectorSink` on `Organism`.
- **Ganglion ports + `stimulate`:** derived input/output ports; perturb-and-settle returns `GanglionResponse`.
- **DNA enzymes:** `compile()` step 3 resolves cascades onto `Genome`; `CascadeTransducer::from_genome` is the primary constructor.
- Hygiene: fixture gene `effector` → `downstream`; homeostasis prose uses “corrective step” for the control-loop actuator.

## 0.2.0 — unreleased

- Rename DNA-layer types to biological vocabulary: `Hyperedge`→`Cistron`,
  `HyperedgeKind`→`CistronKind`, `SpatialHypergraph`/`Hypergraph`→`Grn`;
  Store methods `*_hyperedge`→`*_cistron`, `*_hypergraph`→`*_grn`;
  error `MalformedHyperedge`→`MalformedCistron`. Separates engine vocabulary
  from infinite-db's. No behaviour, hash, or on-disk-format change. Breaking API.

## 0.1.0

- Gardenable M0–M10 scaffold: settle through wall-move, inspector, benches, props.
