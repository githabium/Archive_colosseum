OCA.ticket.solana.md (v0.1)

日本語: 「観測が始まり、観測が終わる。」 — Observation begins, observation ends.

Purpose

OCA Ticket is a Solana account that authorizes a time-boxed observation window for a reader to reconstruct shards (Q-SUD) of a specific NFB under policy. Tickets are verifiable on-chain and compatible with PQC migration.

PDA Seeds
seeds = [ b"oca", nfb_pubkey, reader_pubkey, LE64(epoch_start), LE64(epoch_end) ]

Anchor Layout (sketch)
// 日本語: これは最小形。拡張は別アカウントで。
#[account]
pub struct OcaTicket {
    pub version: u8,              // 0x01
    pub nfb: Pubkey,              // target NFB
    pub reader: Pubkey,           // beneficiary
    pub epoch_start: u64,         // inclusive
    pub epoch_end: u64,           // inclusive
    pub price_lamports: u64,      // required payment (0 for free)
    pub policy_bits: u64,         // bitflags (e.g., POW_REQUIRED=1<<0, NO_EXPORT=1<<1, RATE_LIMIT=1<<2)
    pub issued_at: i64,           // unix seconds
    pub revoked_at: i64,          // 0 if active
    // Signatures (dual-stack during PQC migration)
    pub ed25519_sig: [u8; 64],    // issuer sig (ed25519); verify via syscall
    pub pqc_scheme: u8,           // 0=None,1=Dilithium2,2=Falcon512,...
    pub pqc_sig_hash: [u8;32],    // SHA-256 of full PQC signature stored off-chain or in extension
    pub bump: u8,
}

Policy Bits (suggested)
0: POW_REQUIRED
1: NO_EXPORT (client must zeroize after W)
2: RATE_LIMITED (per-window cap)
3: GEO_MASKED (optional region gating by client attestation)
4: NON_TRANSFERABLE (bound to reader pubkey)

Flows

Issue: Author (or Archive authority) mints PDA with seeds; writes fields; signs ed25519_sig over sha256(ticket_bytes_without_sigs); optionally posts full PQC signature to Arweave/IPFS and stores pqc_sig_hash here.

Verify (on-chain): Program checks seeds, now ∈ [epoch_start, epoch_end], price paid, revoked_at == 0, ed25519 syscall OK. PQC is verified by a companion program or via attested off-chain service; the on-chain check compares pqc_sig_hash.

Revoke: Authority flips revoked_at = now. Clients must treat any further unfolding as invalid; ERL rotation will also invalidate DH automatically after the window.

Events (Anchor)
#[event]
pub struct TicketIssued { pub nfb: Pubkey, pub reader: Pubkey, pub start: u64, pub end: u64, pub price: u64 }
#[event]
pub struct TicketRevoked { pub nfb: Pubkey, pub reader: Pubkey, pub at: i64 }

Storage & Size Notes

Base account is fixed-size (~234 bytes). PQC payloads can be kept in a separate extension account or off-chain with pqc_sig_hash anchoring.

OCA tickets are non-fungible permissions, not tokens. They are PDAs that expire by time and ERL rotation.

Security Notes

Always verify reader is the TX signer (or that the ticket is delegated explicitly).

Price must be enforced by a pre-instruction transfer to the NFB donation PDA.

Clients must zeroize Derivation Hints (DH) after window end.

Auditors should fuzz policy handling and revocation races.

Compatibility

PQC Migration: Initially dual-sign (ed25519 + PQC hash). Once stable PQC verification is available on-chain, enforce PQC-only or hybrid per governance.

Chain-agnostic: The ticket format can be mirrored on other chains; the Envelope hash and ERL remain chain-independent.
