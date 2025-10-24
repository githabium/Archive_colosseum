use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak::{hashv};
use anchor_lang::solana_program::system_program;

declare_id!("ArCh1vECondiNuS111111111111111111111111111");

// Архив — Condinus NFB program (Anchor on Solana)
// Сложная, расширяемая версия: NFB создание, шифрование Condinus-стиля,
// расчет Litrium, донаты на адрес автора (PDA), proof-of-work на возрастание сложности.
//
// 日本語の注釈 — мудрые цитаты о децентрализации и ИИ в комментариях.
// "分散化は自由の形であり、知識は共同の炎だ。" — Децентрализация как огонь.

#[program]
pub mod archive_condinus {
    use super::*;

    pub fn initialize_archive(ctx: Context<InitializeArchive>, bump: u8) -> Result<()> {
        let archive = &mut ctx.accounts.archive;
        archive.author = *ctx.accounts.author.key;
        archive.bump = bump;
        archive.total_chars = 0;
        archive.total_pages = 0;
        archive.litrium_pool = 0;
        archive.difficulty = 16; // base difficulty (bits)
        Ok(())
    }

    /// Create a new NFB record (immutable author link, dynamic pages)
    pub fn create_nfb(ctx: Context<CreateNfb>, title: String, initial_text: String, timestamp: i64, work_proof: u64) -> Result<()> {
        let nfb = &mut ctx.accounts.nfb;
        let archive = &mut ctx.accounts.archive;

        // basic metadata
        nfb.author = archive.author;
        nfb.title = title.clone();
        nfb.created_at = timestamp;
        nfb.total_chars = 0;
        nfb.page_count = 0;
        nfb.litrium_earned = 0;
        nfb.donation_bump = *ctx.bumps.get("donation_account").unwrap_or(&0u8);
        nfb.donation_account = ctx.accounts.donation_account.key();

        // check proof-of-work adapting to archive.total_chars (progressive difficulty)
        let required = compute_threshold(archive.total_chars, archive.difficulty);
        require!(verify_work(&nfb.author, &initial_text, work_proof, required), ArchiveError::InvalidProofOfWork);

        // perform Condinus-style transformation and store encrypted blob
        let transformed = condinus_transform(&initial_text);
        nfb.encrypted = transformed.clone();

        // litrium calculation: L = (S * T) / (E + H)
        // We'll simulate T = time delta (in seconds) since creation of archive (approx),
        // E (core fatigue) and H (heat) are simplified to constants here; onchain they may be updated.
        let s = initial_text.chars().count() as u128;
        let t = 1u128.max((Clock::get()?.unix_timestamp - archive.created_at.unwrap_or(Clock::get()?.unix_timestamp)) as u128);
        let e = 1u128; // core fatigue (placeholder)
        let h = 1u128; // heat (placeholder)
        let lit = (s * t) / (e + h);

        // update counts
        nfb.total_chars = s as u64;
        nfb.page_count = 1;
        nfb.litrium_earned = lit as u64;

        archive.total_chars = archive.total_chars.checked_add(nfb.total_chars).unwrap_or(archive.total_chars);
        archive.total_pages = archive.total_pages.checked_add(1).unwrap_or(archive.total_pages);
        archive.litrium_pool = archive.litrium_pool.checked_add(nfb.litrium_earned).unwrap_or(archive.litrium_pool);

        Ok(())
    }

    /// Append a page to existing NFB. Each new page increases the "difficulty" (litrium harder to mine).
    pub fn append_page(ctx: Context<AppendPage>, page_text: String, timestamp: i64, work_proof: u64) -> Result<()> {
        let nfb = &mut ctx.accounts.nfb;
        let archive = &mut ctx.accounts.archive;

        // progressive difficulty: required threshold grows with total_chars
        let required = compute_threshold(archive.total_chars, archive.difficulty);
        require!(verify_work(&nfb.author, &page_text, work_proof, required), ArchiveError::InvalidProofOfWork);

        // transform new page
        let new_blob = condinus_transform(&page_text);

        // Merge encrypted data (concatenate) and then re-shuffle to increase entropy
        let mut merged = nfb.encrypted.clone();
        merged.extend(new_blob.iter());
        merged = fisher_yates_shuffle(&merged);
        nfb.encrypted = merged;

        // litrium update: diminishing returns — per additional chars, less litrium
        let s = page_text.chars().count() as u128;
        let t = 1u128.max((Clock::get()?.unix_timestamp - nfb.created_at) as u128);

        // diminishing: multiplier = 1 / (1 + log2(1 + archive.total_chars)) approximated as integer
        let denom = 1u128 + integer_log2(1u128 + archive.total_chars as u128);
        let lit = (s * t) / denom;

        let lit_u64 = lit.min(u128::from(u64::MAX)) as u64;

        nfb.total_chars = nfb.total_chars.checked_add(s as u64).unwrap_or(nfb.total_chars);
        nfb.page_count = nfb.page_count.checked_add(1).unwrap_or(nfb.page_count);
        nfb.litrium_earned = nfb.litrium_earned.checked_add(lit_u64).unwrap_or(nfb.litrium_earned);

        archive.total_chars = archive.total_chars.checked_add(s as u64).unwrap_or(archive.total_chars);
        archive.total_pages = archive.total_pages.checked_add(1).unwrap_or(archive.total_pages);
        archive.litrium_pool = archive.litrium_pool.checked_add(lit_u64).unwrap_or(archive.litrium_pool);

        // slowly raise global difficulty
        if archive.total_chars % 1024 == 0 {
            archive.difficulty = archive.difficulty.saturating_add(1);
        }

        Ok(())
    }

    /// Withdraw donations from PDA to author — only author can call
    pub fn claim_donations(ctx: Context<ClaimDonations>) -> Result<()> {
        let donation_acc = &mut ctx.accounts.donation_account;
        let author = &ctx.accounts.author;
        let balance = donation_acc.to_account_info().lamports();
        **donation_acc.to_account_info().try_borrow_mut_lamports()? -= balance;
        **author.to_account_info().try_borrow_mut_lamports()? += balance;
        Ok(())
    }
}

// ---------- Contexts and Accounts ----------

#[derive(Accounts)]
#[instruction(bump:u8)]
pub struct InitializeArchive<'info> {
    #[account(init, payer = author, space = 8 + Archive::MAX_SIZE)]
    pub archive: Account<'info, Archive>,
    #[account(mut)]
    pub author: Signer<'info>,
    /// CHECK: donation PDA will be derived; not used here
    #[account(mut)]
    pub donation_account: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateNfb<'info> {
    #[account(mut, has_one = author)]
    pub archive: Account<'info, Archive>,
    #[account(init, payer = payer, space = 8 + Nfb::MAX_SIZE)]
    pub nfb: Account<'info, Nfb>,
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: derived PDA used as donation vault for this NFB
    #[account(
        mut,
        seeds = [b"donation", nfb.key().as_ref()],
        bump
    )]
    pub donation_account: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AppendPage<'info> {
    #[account(mut, has_one = author)]
    pub archive: Account<'info, Archive>,
    #[account(mut, has_one = author)]
    pub nfb: Account<'info, Nfb>,
    pub author: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimDonations<'info> {
    #[account(mut, has_one = author)]
    pub nfb: Account<'info, Nfb>,
    #[account(mut, seeds = [b"donation", nfb.key().as_ref()], bump = nfb.donation_bump)]
    pub donation_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub author: Signer<'info>,
}

// ---------- Data Structures ----------

#[account]
pub struct Archive {
    pub author: Pubkey,
    pub bump: u8,
    pub total_chars: u64,
    pub total_pages: u64,
    pub litrium_pool: u64,
    pub difficulty: u8,
    pub created_at: Option<i64>,
}

impl Archive {
    pub const MAX_SIZE: usize = 32 + 1 + 8 + 8 + 8 + 1 + 9;
}

#[account]
pub struct Nfb {
    pub author: Pubkey,
    pub title: String,
    pub created_at: i64,
    pub total_chars: u64,
    pub page_count: u64,
    pub litrium_earned: u64,
    pub donation_account: Pubkey,
    pub donation_bump: u8,
    pub encrypted: Vec<u8>, // Condinus blob
}

impl Nfb {
    // generous sizing for open-source extension
    pub const MAX_SIZE: usize = 32 + (4 + 256) + 8 + 8 + 8 + 32 + 1 + (4 + 4096);
}

// ---------- Utilities / Cryptographic-ish functions ----------

// Simulate Condinus translation -> noise -> shuffle.
// Note: Onchain we cannot call real translation APIs; instead we deterministically mix the text
// with 17 language markers using hashing to generate a multi-lingual fused blob.

fn condinus_transform(input: &str) -> Vec<u8> {
    // Japanese comment: "言葉は魂の器だ。コードはその容れ物。"
    // 1) Produce 17 deterministic language variants by hashing input with language ids
    let languages: [&str; 17] = [
        "en","es","fr","de","ru","ja","zh","ar","hi","pt","it","nl","sv","no","fi","ko","tr",
    ];

    let mut acc: Vec<u8> = Vec::new();
    for (i, lang) in languages.iter().enumerate() {
        let tag = format!("|{}|{}|", lang, i);
        let mid = format!("{}{}{}", input, tag, i);
        let h = hashv(&[mid.as_bytes()]);
        acc.extend_from_slice(&h.0);
    }

    // 2) Overlay noise derived from a PRNG seeded by input hash
    let seed = hashv(&[input.as_bytes()]).0;
    let noise = deterministic_xor_noise(&acc, &seed);

    // 3) Fisher-Yates shuffle for final chaos
    let mut shuffled = fisher_yates_shuffle(&noise);
    shuffled
}

fn deterministic_xor_noise(data: &Vec<u8>, seed: &[u8;32]) -> Vec<u8> {
    // Generate pseudo-random stream by chaining keccak hashes
    let mut out = data.clone();
    let mut state = seed.to_vec();
    for i in 0..out.len() {
        let h = hashv(&[&state, &(i as u64).to_le_bytes()]);
        out[i] ^= h.0[i % 32];
        // update state
        state = h.0.to_vec();
    }
    out
}

// Fisher-Yates shuffle deterministic variant using keccak tree
fn fisher_yates_shuffle(input: &Vec<u8>) -> Vec<u8> {
    let mut out = input.clone();
    let n = out.len();
    if n <= 1 { return out; }
    for i in (1..n).rev() {
        let h = hashv(&[&(i as u64).to_le_bytes(), &out]);
        let j = (u64::from_le_bytes(first_8(&h.0)) as usize) % (i+1);
        out.swap(i, j);
    }
    out
}

fn first_8(slice: &[u8]) -> [u8;8] {
    let mut a = [0u8;8];
    for i in 0..8 { a[i] = slice[i]; }
    a
}

// Progressive difficulty: compute threshold from total_chars and base bits
fn compute_threshold(total_chars: u64, base_bits: u8) -> u128 {
    // difficulty grows logarithmically with total_chars
    let extra = integer_log2(1u128 + total_chars as u128) as u8;
    let bits = base_bits.saturating_add(extra);
    // threshold as max hash value >> bits
    if bits >= 127 { return 0; }
    (u128::MAX) >> bits
}

fn verify_work(author: &Pubkey, text: &str, proof: u64, threshold: u128) -> bool {
    let seed = [&author.to_bytes()[..], text.as_bytes(), &proof.to_le_bytes()].concat();
    let h = hashv(&[&seed]);
    // take first 16 bytes as u128
    let mut val_bytes = [0u8;16];
    for i in 0..16 { val_bytes[i] = h.0[i]; }
    let val = u128::from_le_bytes(val_bytes);
    val <= threshold
}

// integer base-2 log (floor)
fn integer_log2(mut x: u128) -> u128 {
    let mut r = 0u128;
    while x > 1 {
        x >>= 1;
        r += 1;
    }
    r
}

// ---------- Errors ----------

#[error_code]
pub enum ArchiveError {
    #[msg("Invalid proof-of-work provided; please compute stronger work.")]
    InvalidProofOfWork,
}

// ---------- End of program ----------

// Japanese sign-off comment: "このコードは道具だ。しかし道具は詩にもなり得る。"
