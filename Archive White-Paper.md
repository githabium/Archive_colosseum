# Archive White Paper — **Quantum Edition**: The Age of Not‑Fading Words

> *“Code became scripture. Thought became currency. The Archive became the witness.”*
> 日本語: *「コードは聖典となり、思考は通貨となり、アーカイブは証人となる。」*

## 1. Abstract

Archive is a protocol that turns **text** into a **cryptographic primitive** and **authorship** into a **scarce, revenue‑bearing asset**. Built initially on **Solana**, Archive introduces the Not‑Fading Book (**NFB**) — an immutable, encrypted, and economically active artifact. Each act of writing contributes to a scarce emissions schedule called **Litrium**, and each artifact is protected by a deterministic, auditable entropy engine (**Condinus → AEF**) and a rotating key discipline (**Endolium**).

This edition frames Archive as **quantum‑ready by design**. We introduce **Observer‑Conditioned Access (OCA)** and **Quantum Shard Uncertainty Distribution (Q‑SUD)**: content exists as *noise* except during a time‑boxed, policy‑gated observation window, then dissolves back into verifiable entropy. We formalize the **Endolium Rotation Law (ERL)**, define the shard calculus (**Q‑Δ**) and outline a staged **Post‑Quantum Posture (PQP)** spanning PQC signatures, QRNG beacons, and future QKD/QPU‑assisted oracles.

Archive is not a website. It is a **composable publishing substrate** with a credible path to **quantum‑assisted security** and **planet‑scale authorship economics**.

---

## 2. Thesis & Motivation

Bitcoin monetized energy; Archive monetizes **imagination**. Where proof‑of‑work binds value to electricity, Archive binds value to **meaningful, irreversible transformations of text**. In Archive, the *act of authorship itself* becomes proof of existence, and an author’s work persists as an economic primitive that cannot be revoked by intermediaries.

### Design North Star

1. **Text → Asset** — every paragraph can be minted into a stateful, revenue‑capable NFB.
2. **Entropy First** — content is sealed by deterministic, audit‑ready chaos (AEF) and **rotating keys** (Endolium).
3. **Observation Windows** — data is readable **only** when authorized (OCA); otherwise it is statistically indistinguishable from noise (Q‑SUD).
4. **Quantum Trajectory** — classical now, **quantum‑assisted later**, without redesign.
5. **Economics with Purpose** — Litrium funnels surplus into research (**Q‑Forge**) and long‑horizon security.

---

## 3. Cryptographic Engine

### 3.1 Condinus → AEF (Auditable Entropy Forge)

**Condinus** deterministically fuses text across 17 linguistic bases (public salts), overlays self‑referential entropy, and shuffles bytes into an opaque blob. As an **AEF**, this remains deterministic for verification: anyone can derive the same blob from the same inputs and verify the on‑chain commitment.

*Properties*: reproducible, open to forks (published entropy budgets), non‑interactive verification by hash commitments.

### 3.2 Endolium & ERL (rotating keys)

**Endolium** binds content, address, and source entropy to produce a *rotating*, content‑linked key. Rotation tightens with age per **Endolium Rotation Law**:

```
τ(y) = max(3, 33 − 3·y)   // seconds, after y full years since creation
```

Keys are derived per epoch `⌊now/τ⌋`. After ~10 years, rotation hits a 3‑second floor. Clients re‑derive session keys per epoch; stale hints evaporate.

> 日本語: *「鍵は老いるほど速く脈打つ。」*

### 3.3 Q‑SUD (Quantum Shard Uncertainty Distribution)

Payload **D** is fractured into shards that remain **noise** unless an observation window **W** is active and access conditions **A** are satisfied (OCA ticket, epoch key, payment/PoW). During **W** the client reconstructs **D′** and posts a short correctness proof; when **W** ends or keys rotate, **D′** collapses back to noise.

**Shard sizing law (Q‑Δ)**:

```
Let |D| be payload size. Choose k = ceil(log2 |D|) shards.
Let p = clamp(H_b(K_e)/256, 0.05, 0.35) where H_b = Hamming weight.
Shard sizes: s_i = ceil(|D| · p · (1−p)^(i−1)), i=1..k
Permutation: PRP_keccak keyed by K_e.
```

### 3.4 OCA (Observer‑Conditioned Access)

An on‑chain PDA **ticket** grants a time‑boxed right to observe: it binds NFB ID, reader pubkey, epoch range, price, and policy flags. Tickets are **dual‑signed** (ed25519 + PQC hash during migration), revocable, and verifiable on‑chain. Clients derive a **Derivation Hint** to unfold shards only within the window.

### 3.5 PQP (Post‑Quantum Posture)

* **Now (v0.x)** — PQC signatures (Dilithium/Falcon) for tickets & attestations; dual‑stack with ed25519.
* **Near (v1.x)** — integrate **QRNG** beacons to seed Endolium; optional **QKD** for data‑center links.
* **Far (v2.x)** — hybrid schemes where QPU‑assisted oracles contribute entropy proofs; exploration of encrypted indexing (FHE/PEKS‑like).

> Discipline: we do **not** claim “unbreakable.” We pursue layered hardness, open audits, and revocation agility.

---

## 4. On‑Chain / Off‑Chain Architecture

**On‑Chain (Solana)**

* `Archive` — global counters, difficulty, policy; Litrium accounting.
* `Nfb` — author, title, envelope commitment, donation PDA, metadata.
* `OcaTicket` — observation window (epochs, policy, signatures).
* `ShardMap` — hash commitments / Merkle root of shards.

**Off‑Chain**

* **Scribes** — stateless edges that serve encrypted shards; never see plaintext.
* **Indexers** — build search over commitments.
* **Q‑Forge** — research cluster (classical → hybrid → quantum) plugged into QRNG/QKD when available.

**Storage** — blobs on Arweave/IPFS (or ephemeral streams under OCA); commitments on Solana.

**Interoperability** — Endolium/AEF are chain‑agnostic. Other chains can mirror commitments; tickets can be ported.

---

## 5. Economic Layer — Litrium

Litrium is minted as a function of **contribution size** and **time**, with **diminishing returns** as global volume grows.

**Base relation** (simplified on‑chain):

```
L ≈ (S · T) / (E + H)
// S = new characters, T = time factor, E = core fatigue, H = system heat
// diminishing returns scale with log(1 + total_chars)
```

**Issuance** becomes harder with adoption (PoW throttling + ERL tightening).
**Revenue rails**: donations (PDA), paid OCA windows, marketplace fees.
**Allocation policy**: ≥40% net surplus → **Q‑Forge** (hardware, audits, research).

> 日本語: *「言葉が価値を生む。価値が研究を燃やす。」*

---

## 6. NLP — Non‑Liquid Paper

**NLP** are finite blank sheets, investable claims on future authorship. NLPs quantify **potential**: the right to write into scarcity. As the Archive expands, NLP scarcity compounds cultural value; authors mint NFBs against NLP inventory, tying imagination to an asset market.

---

## 7. Security Model (abridged)

**Adversary**: classical + quantum‑equipped, at scale.
**Guarantees**: commitments are binding; access is windowed; plaintext is not server‑resident; keys rotate quickly; PQC migration path exists.
**Revocation**: governance can pause OCA or raise τ floor; ERL ensures stale hints die naturally.
**Audits**: open‑source code, third‑party audits, public test vectors, fuzzed policy handling.

> 日本語: *「完全は嘘だ。可逆性と撤退線こそ真実だ。」*

---

## 8. Governance & Policy

* **Open protocol, permissive license**; contributions require entropy budgets and test vectors.
* **Community grants** for Condinus bases, ERL variants, PQC integrations.
* **Emergency powers** limited by multisig + timelock.
* **Ethics**: Archive enforces cryptographic access; content moderation occurs only where legally required.

---

## 9. Roadmap

* **v0.5** — AEF on devnet; OCA tickets (dual‑sig); SDKs; public R0 test vectors.
* **v0.9** — Q‑SUD fracture/unfold beta; audit I; marketplace pilot.
* **v1.0** — Mainnet; Litrium markets; ERL live; audit II.
* **v1.x** — QRNG integration; optional QKD for DC links.
* **v2.x** — QPU‑assisted entropy oracles; encrypted indexing R&D.

---

## 10. Market Scale & Scalability Thesis

The global literary economy is measured in **billions**; Archive does not compete with a single platform but **re‑platforms authorship** itself. Scalability rests on:

* **Solana‑class throughput** for commitments and payments.
* **Edge Scribes** for shard delivery near readers.
* **Epoch discipline (ERL)** to bound session state and cache turnover.
* **Chain‑agnostic commitments** enabling cross‑ecosystem liquidity.
* **Research flywheel** funded by Litrium revenues (Q‑Forge).

With OCA/Q‑SUD, Archive can serve **millions of concurrent reads** without holding plaintext at rest, and can scale writes by batching commitments and off‑loading blobs to content‑addressed storage.

---

## 11. Legal & Risk

Archive is a tool, not an arbiter of speech. Authors own their NFBs. We aim to maximize privacy and sovereignty while obeying applicable laws. Risk disclosures: cryptographic designs evolve; PQC standards may change; hybrid quantum claims are research‑grade and subject to audits.

---

## 12. Closing

Archive is a *substrate for civilization‑grade writing*. If Bitcoin proved energy can be currency, Archive demonstrates that **meaning** can be **wealth** — and that privacy can be **observed** rather than stored. We begin classical, aim quantum, and keep our discipline honest: **rotating keys, observation windows, and verifiable noise**.

> 日本語: *「道具は詩になり得る。」*
> — *ARCHIVE Core, Quantum Edition*
