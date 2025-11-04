# Archive — Solana Colosseum Technical Brief (v0.1)

**Purpose:** a concise, engineer‑first overview of Archive for Solana Colosseum reviewers: what problem we solve, why Solana, what’s implemented, how it works on‑chain, and what feedback we seek.

---

## TL;DR

* **Archive** turns text into a cryptographic asset (**NFB** — Not‑Fading Book) with revenue rails.
* **Why Solana:** Sealevel parallelism, low fees, PDAs, mature tooling (Anchor), ed25519 syscall — ideal for high‑rate authorship commits + micropayments.
* **Core primitives:** **Endolium** (rotating content‑bound keys), **AEF** (auditable entropy from Condinus), **OCA** tickets (observer‑windowed access), **Q‑SUD** (sharded readability), **Litrium** (issuance tied to writing).
* **Status:** specs + reference code skeletons ready; devnet rollout planned (NFB + OCA). PQC dual‑sig path defined.
* **Ask:** compute/rent review, PDA design sanity check, batching patterns, fee markets/prioritization guidance.

---

## Why Solana (engineer view)

* **Throughput & costs:** meets Archive’s write‑heavy pattern (page commits, tips, tickets) without pricing‑out users.
* **Parallelism:** Sealevel maps well to independent NFB operations and shard map updates.
* **PDAs:** donation vaults, OCA tickets, shard maps — clean derivation and access control.
* **Tooling:** Anchor, program‑test, ed25519 syscall, good local/devnet UX for rapid iteration.

---

## What’s different

* **NFB ≠ NFT:** stateful, growing cryptographic artifact with on‑chain commitments + off‑chain encrypted shards.
* **Access by observation:** OCA grants time‑boxed readability; outside the window content is verifiable noise (Q‑SUD).
* **Rotating keys:** Endolium enforces **ERL** `τ(y) = max(3, 33−3·y)`; session hints expire naturally.
* **Economics:** Litrium issuance gets harder with global usage (log‑scaled), aligning incentives with meaningful writing.

---

## Program Map (initial wave)

1. **archive_condinus**

   * Accounts: `Archive`, `Nfb` (+ PDA `donation`), optional `ShardMap` (Merkle root).
   * IX: `initialize_archive`, `create_nfb(title, initial_text, ts, work_proof)`, `append_page(page_text, ts, work_proof)`, `claim_donations`.
   * Notes: PoW verify (`keccak(author||text||nonce) ≤ threshold(total_chars)`), commitments only; blobs to Arweave/IPFS.

2. **oca_ticket**

   * Account: `OcaTicket` (PDA over `nfb`, `reader`, epoch range).
   * IX: `issue_ticket`, `revoke_ticket`.
   * Policy bits: POW_REQUIRED, NO_EXPORT, RATE_LIMITED, NON_TRANSFERABLE.
   * Signatures: ed25519 syscall + `pqc_sig_hash` (PQC kept in extension/off‑chain; migration ready).

> Detailed layouts in `SPEC.endolium.md` and `OCA.ticket.solana.md`.

---

## Key Flows

**Author → Create NFB**

```
1) Client computes PoW for initial_text.
2) IX create_nfb() stores envelope/commitments; mints donation PDA.
3) UI shows NFB page + tip button (SOL → PDA).
```

**Append Page**

```
1) Client PoW for page_text.
2) IX append_page() updates commitment; adjusts Litrium counters; optional difficulty bump.
```

**Reader → Observe**

```
1) Reader buys/receives OCA ticket (epochs, price, policy).
2) Client derives Derivation Hint; unfolds shards only within window.
3) After window or rotation, DH invalid; must re‑observe.
```

---

## Compute, Rent, Storage (targets)

* **create_nfb:** < 150k CU (PoW verify + PDA init + hash commits).
* **append_page:** < 120k CU (verify + commit update).
* **issue_ticket:** < 70k CU (PDA init + ed25519 verify).
* **Accounts:** `Nfb` ~ 1.5–3 KB (metadata + pointers), `OcaTicket` ~ 200–250 B base.
* **Storage:** blobs off‑chain (Arweave/IPFS). On‑chain keeps content‑addressed hashes / Merkle roots.

> We will instrument CU via program‑test + devnet and publish real metrics with benches.

---

## Security & PQC Posture

* **Layered:** commitments binding; session‑window access; Endolium rotation; PoW throttle.
* **PQC migration:** dual‑sig (ed25519 + PQC hash) now; swap to on‑chain PQC verify when available.
* **Revocation:** `revoke_ticket` + ERL rotation ensures stale access dies.
* **Audits:** public test vectors (R0), fuzzed policies, third‑party audit pre‑mainnet.

---

## Dev Quickstart (draft)

```bash
# deps (example)
rustup default stable
cargo install --locked anchor-cli
solana config set --url https://api.devnet.solana.com

# build & test
anchor build
anchor test

# deploy (devnet)
anchor deploy
```

Artifacts: program IDs, IDLs, and TS SDK helpers (`deriveEpoch`, `deriveEndoliumKey_R0`, `prepareOcaTicketIx`) will be published in `/sdk`.

---

## Milestones & KPIs

* **M0 (devnet POC):** NFB create/append; OCA issue/verify; ≥ 100 NFBs; ≥ 1k OCA windows.
* **M1 (beta):** public readers; tips; > 10k TX; compute budget report; audit I.
* **M2 (mainnet):** ERL live; Litrium markets; > 50k TX; audit II; PQC path locked.

---

## What Feedback We Want from Solana Devs

* Compute/rent: realistic CU ceilings and rent sizing for `Nfb`/`OcaTicket`.
* PDA patterns: donation vault + ticket seeds — any pitfalls?
* Fee markets: guidance on priority fees for bursty writes (append waves).
* Sealevel patterns: batching/parallelizing NFB updates; any anti‑patterns spotted.
* Security model: syscall usage, replay risks, revocation races.

---

## Contact & Refs

* **Specs:** `Archive White Paper — Quantum Edition`, `SPEC.endolium.md`, `OCA.ticket.solana.md`.
* **Code:** `programs/archive_condinus`, `programs/oca_ticket`, `sdk/` (TS helpers).
* **Team:** Archive Core — reachable via repo issues/PRs.

> We built Archive to feel inevitable on Solana: high‑rate authorship commits, windowed readability, and a clean path to post‑classical security. Looking forward to your scrutiny and suggestions.
