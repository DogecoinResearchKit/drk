//! Dogeview's display text over Bitview's: names, units, chain facts and the brand color,
//! applied under the `dogecoin` feature to the showcases at build time, the API's descriptions and
//! the clients' display names. Identifiers (series names, unit keys, module paths, pool names) stay
//! as they are, so each rule guards what surrounds its match.

/// The Bitview orange, without its closing parenthesis so alpha variants (`/ 0.1)`) match too.
const BRAND_ORANGE: &str = "oklch(64.34% 0.191 44.41";
/// Dogecoin gold (#C2A633).
const BRAND_GOLD: &str = "oklch(72.9% 0.132 94.8";

type Guard = fn(&str, &str) -> bool;

/// `(from, to, guard)`, in order; the guard sees the text before and after the match.
const RULES: &[(&str, &str, Guard)] = &[
    (BRAND_ORANGE, BRAND_GOLD, any),
    ("#e65b00", "#c2a633", any),
    (
        "oklch%2864.34%25 0.191 44.41",
        "oklch%2872.9%25 0.132 94.8",
        any,
    ),
    // Text on the page keeps its contrast in a darker gold.
    ("oklch(52% 0.16 44.41", "oklch(52% 0.11 94.8", any),
    ("#f7931a", "#d4b84a", any),
    ("#a95600", "#8a7420", any),
    ("#c24e00", "#a68b22", any),
    ("#a94400", "#7d681c", any),
    ("rgb(230 91 0", "rgb(194 166 51", any),
    // Pages' accents as light and dark pairs: the same colors elsewhere are data's, not the brand's.
    (
        "light-dark(#d4570a, #f2873a)",
        "light-dark(#8a7420, #d9bf55)",
        any,
    ),
    (
        "light-dark(#9c3d00, #ffb27a)",
        "light-dark(#6e5c17, #ecd88f)",
        any,
    ),
    (
        "light-dark(#fbe9dc, #33221a)",
        "light-dark(#f8f2dc, #2e2914)",
        any,
    ),
    (
        "light-dark(#f6d7bf, #47291a)",
        "light-dark(#efe3b8, #453c1a)",
        any,
    ),
    (
        "light-dark(#ad470b, #ff8a3d)",
        "light-dark(#8a7420, #d9bf55)",
        any,
    ),
    (
        "light-dark(#fff8f2, #170b02)",
        "light-dark(#fffcf0, #151204)",
        any,
    ),
    (
        "light-dark(rgb(184 68 10), rgb(255 128 52))",
        "light-dark(rgb(138 116 32), rgb(217 191 85))",
        any,
    ),
    // Oracle's heat ramp, low to high, and its launch color.
    (
        "light-dark(rgb(250 230 212), rgb(54 16 4))",
        "light-dark(rgb(248 241 214), rgb(40 33 6))",
        any,
    ),
    (
        "light-dark(rgb(245 172 112), rgb(196 66 14))",
        "light-dark(rgb(232 210 128), rgb(150 124 30))",
        any,
    ),
    (
        "light-dark(rgb(232 108 30), rgb(255 152 66))",
        "light-dark(rgb(194 166 51), rgb(230 200 90))",
        any,
    ),
    ("data-color=\"#e86c1e\"", "data-color=\"#c2a633\"", any),
    ("oklch(67.64% 0.191 44.41", "oklch(76% 0.132 94.8", any),
    ("https://bitview.dev/", "https://dogeview.space/dev/", any),
    ("https://bitview.dev", "https://dogeview.space/dev/", any),
    (
        "https://bitcoinresearchkit.org/",
        "https://dogeview.space/brk/",
        any,
    ),
    (
        "https://bitcoinresearchkit.org",
        "https://dogeview.space/brk/",
        any,
    ),
    ("bitview.space", "dogeview.space", any),
    ("sats: \"sats\"", "sats: \"koinu\"", any),
    ("Bitview", "Dogeview", word),
    ("Bitcoin", "Dogecoin", prose_word),
    ("bitcoin", "dogecoin", lower_prose_word),
    ("bitcoind", "dogecoind", lower_prose_word),
    ("bitcoin-cli", "dogecoin-cli", lower_prose_word),
    ("BTC", "DOGE", prose_word),
    ("sats/vB", "koinu/vB", word_start),
    ("sat/vB", "koinu/vB", word_start),
    ("sat/PH", "koinu/PH", word_start),
    ("₿", "Ð", any),
    ("Satoshis", "Koinu", word),
    ("satoshis", "koinu", word),
    ("Sats per dollar", "Koinu per dollar", word),
    ("sats", "koinu", amount_unit),
    ("Sats", "Koinu", amount_unit),
    ("\" sat\"", "\" koinu\"", any),
    ("} sat`", "} koinu`", any),
];

/// Bitcoin's chain facts in prose, and Dogecoin's, applied in order before the rules. Any
/// whitespace in `from` matches any run of it, as doc comments wrap where they like.
const FACTS: &[(&str, &str)] = &[
    // Dogecoin enforced BIP30 from genesis: it has no duplicate coinbases.
    (
        ", and the coinbases of blocks 91812 and 91722 once duplicates at 91842 and 91880 overwrite them;",
        ";",
    ),
    (
        ", plus the coinbase outputs of blocks 91812 and 91722, which duplicates at 91842 and 91880 overwrote.",
        ".",
    ),
    (
        "unclaimed block rewards and the two coinbase outputs that BIP30 duplicates overwrote (blocks 91812 and 91722, at 91842 and 91880)",
        "and unclaimed block rewards",
    ),
    (
        "https://github.com/bitcoinresearchkit/mono",
        "https://github.com/DogecoinResearchKit/drk",
    ),
    // Dogecoin's prices: every block at its day's exchange close, the earliest carried back to
    // genesis; there is no on-chain oracle.
    (
        "Heights before 340,000 use baked historical exchange prices; later heights use an on-chain oracle that estimates price from round-USD transaction-output patterns.",
        "Each block is priced at its day's DOGE/USD exchange close.",
    ),
    (
        "This is a model-derived block price, not a contemporaneous exchange ticker.",
        "This is a daily exchange close, not a contemporaneous ticker.",
    ),
    (
        "from exchanges' history at first, then estimated from the chain itself.",
        "each at its day's exchange close.",
    ),
    (
        "Before the first exchange price (block 74,641, August 2010), this price and USD amounts valued with it are zero.",
        "Blocks before the first exchange close carry the earliest one.",
    ),
    (
        "Zero before the first exchange price (block 74,641, August 2010).",
        "Blocks before the first exchange close carry the earliest one.",
    ),
    (
        "Blocks before the first price (block 74,641) are left out: the period holding it starts there, and earlier periods are flat zero candles.",
        "Blocks before the first exchange close carry the earliest one.",
    ),
    ("block 74,641, August 2010", "block 0, December 2013"),
    ("block 74,641", "block 0"),
    (
        "the first priced interval (2010-08-16)",
        "the first priced interval (2013-12-06)",
    ),
    // A price's unit is the micro-dollar (`CENTS_PER_DOLLAR` is 1,000,000), not the cent.
    ("of the price in cents", "of the price in micro-dollars"),
    ("rounded to the cent", "rounded to the micro-dollar"),
    ("half a cent", "half a micro-dollar"),
    // The oracle's endpoints: only its price answers, from the tip's exchange close.
    (
        "On-chain BTC/USD price derived purely from round-dollar payment patterns in transaction outputs, with no external price feed. Payment activity is binned on a log scale, and a smoothed EMA over recent blocks locates the price.",
        "Dogeview has no on-chain oracle: each block is priced at its day's DOGE/USD exchange close (CryptoCompare, Bitfinex and Coinbase).",
    ),
    (
        "The live price is at `/api/oracle/price`, confirmed per-height price history at `/api/series/price/height`. How it read each block (what it counted, where it looked, what it found) is at `/api/oracle/blocks/{height}`, and now at `/api/oracle/live`; the payments behind its prices, 144 blocks at a time, at `/api/oracle/payments/{start}`.",
        "The latest close is at `/api/oracle/price`, per-height prices at `/api/series/price/height`; the oracle's readings (`/api/oracle/blocks/{height}`, `/api/oracle/live`, `/api/oracle/payments/{start}`) are not found.",
    ),
    (
        "Current BTC/USD price in dollars, derived from on-chain round-dollar output patterns in the oracle's recent-block window, with the mempool's outputs blended in when available, to the cent: `price` of `GET /api/oracle/live`. Confirmed per-height history is available at `GET /api/series/price/height`. Not found before the oracle's first prices.",
        "Current DOGE/USD price in dollars: the exchange close of the tip block's day. Per-height history is at `GET /api/series/price/height`.",
    ),
    (
        "The oracle's reading now, as `GET /api/oracle/blocks/{height}` reads a block: with the mempool's outputs blended in as the next block (`mempool` true, `height` the next block's, `outputs` the mempool's), its search from the tip's answer, `price` the live price; without a mempool at the tip, the tip block's own. One snapshot: its parts never from two moments, `hash` the confirmed tip it was read at. Not found before the oracle's first prices, nor before height 508,000.",
        "Not found on Dogeview, which has no on-chain oracle: each block is priced at its day's DOGE/USD exchange close.",
    ),
    (
        "How the oracle read a confirmed block, after height 508,000: what it counted (the smoothed histogram of the payments of the block and the 11 before it, by amount on its log scale; the block's outputs as its filter judged them, each skipped one under the first rule that skips it), where it looked (every candidate bin in its window, 12 below the block before's answer to 11 above, with its score) and what it found (the bin it picked and its price). Bins: 200 a decade, a bin `round(log10(sats) * 200)`; as where $100 lands, a bin is the price `10^(10 - bin / 200)` dollars. A settled block's never changes.",
        "Not found on Dogeview, which has no on-chain oracle: each block is priced at its day's DOGE/USD exchange close.",
    ),
    ("The oracle's payments, 144 blocks at a time", "The oracle's payments"),
    (
        "For each of the 144 blocks from `start` (a multiple of 144, else a bad request; from 508,032, earlier ones out of range): its payments as the oracle filters them, each read as each of its 19 round-dollar amounts ($1 to $10,000) at its exact amount in sats, counted by the price each reading implies, in buckets a tenth of a bin wide (0.115% of price) over the block's search window: finer than the oracle's bins, the evidence behind its prices. The chunk at the tip holds its blocks so far, `hash` its last block's; a whole, settled chunk never changes.",
        "Not found on Dogeview, which has no on-chain oracle: each block is priced at its day's DOGE/USD exchange close.",
    ),
    // Merge-mined blocks (from 371,337) carry an AuxPoW proof after the header; none has witness
    // data.
    (
        "including witness data: its 80-byte header, transaction-count CompactSize, and every serialized transaction.",
        "as stored: its 80-byte header, the AuxPoW proof of a merge-mined block (from 371,337), the transaction-count CompactSize, and every serialized transaction.",
    ),
    (
        "the bytes each block takes, witness data included.",
        "the bytes each block takes, AuxPoW proof included.",
    ),
    // Block timing: a minute a block, 1,440 a day.
    (
        "the 144 blocks expected at Bitcoin's ten-minute target",
        "the 1,440 blocks expected at Dogecoin's one-minute target",
    ),
    (
        "proved per ten-minute target",
        "proved per one-minute target",
    ),
    ("at the ten-minute target", "at the one-minute target"),
    ("at the 10-minute target", "at the one-minute target"),
    ("one block every ten minutes", "one block every minute"),
    ("at ten minutes a block", "at a minute a block"),
    // Difficulty: retargeted every block by DigiShield since 145,000; Dogeview's epochs are
    // 240-block windows.
    (
        "adjusted every 2,016 blocks to keep them ten minutes apart",
        "adjusted every block (DigiShield, since block 145,000) to keep them a minute apart",
    ),
    ("2,016", "240"),
    // Halvings: every 100,000 blocks through 600,000 (February 2015), a fixed 10,000 DOGE since.
    (
        "one every 210,000 blocks.",
        "one every 100,000 blocks through 600,000, a fixed reward since.",
    ),
    ("divided by 210,000", "divided by 100,000"),
    (
        "210,000 minus height modulo 210,000",
        "100,000 minus height modulo 100,000",
    ),
    // The rarity meter's window: four years of blocks, as Bitcoin's 210,000 are.
    (
        "210,000-block backward half-life",
        "2,102,400-block backward half-life",
    ),
    ("from 210,000 of them", "from 2,102,400 of them"),
    ("most recent 210,000", "most recent 2,102,400"),
    ("height 210,000", "height 2,102,400"),
];

/// `text` with every fact and rule applied.
pub fn relabel(text: &str) -> String {
    let text = FACTS.iter().fold(text.to_owned(), |text, &(from, to)| {
        replace_loose(&text, from, to)
    });
    RULES.iter().fold(text, |text, &(from, to, guard)| {
        replace_guarded(&text, from, to, guard)
    })
}

fn replace_loose(text: &str, from: &str, to: &str) -> String {
    let mut words = from.split_whitespace();
    let first = words.next().unwrap();
    let words: Vec<&str> = words.collect();
    let mut out = String::new();
    let (mut rest, mut search) = (0, 0);
    while let Some(found) = text[search..].find(first) {
        let at = search + found;
        search = at + first.len();
        if let Some(len) = loose_match(&text[search..], &words) {
            out.push_str(&text[rest..at]);
            out.push_str(to);
            search += len;
            rest = search;
        }
    }
    out.push_str(&text[rest..]);
    out
}

/// The length of `words` at the start of `after`, each after a run of whitespace.
fn loose_match(after: &str, words: &[&str]) -> Option<usize> {
    let mut at = 0;
    for word in words {
        let gap = after[at..].len() - after[at..].trim_start().len();
        if gap == 0 || !after[at + gap..].starts_with(word) {
            return None;
        }
        at += gap + word.len();
    }
    Some(at)
}

fn replace_guarded(text: &str, from: &str, to: &str, guard: Guard) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = 0;
    for (at, _) in text.match_indices(from) {
        if at < rest {
            continue;
        }
        let end = at + from.len();
        if guard(&text[..at], &text[end..]) {
            out.push_str(&text[rest..at]);
            out.push_str(to);
            rest = end;
        }
    }
    out.push_str(&text[rest..]);
    out
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn ends_word(before: &str) -> bool {
    before.chars().next_back().is_some_and(is_word)
}

fn starts_word(after: &str) -> bool {
    after.chars().next().is_some_and(is_word)
}

fn any(_: &str, _: &str) -> bool {
    true
}

fn word_start(before: &str, _: &str) -> bool {
    !ends_word(before)
}

fn word(before: &str, after: &str) -> bool {
    !ends_word(before) && !starts_word(after)
}

/// Not an object key (`Bitcoin:`, a unit's name in the API).
fn not_key(after: &str) -> bool {
    !after.starts_with(':')
}

/// Not a pool named after the chain (`Bitcoin.com`).
fn not_pool(after: &str) -> bool {
    ![".com", ".COM", ".TOP", ".top", " Guild", " Pool", " pool"]
        .iter()
        .any(|pool| after.starts_with(pool))
}

fn prose_word(before: &str, after: &str) -> bool {
    word(before, after) && not_key(after) && not_pool(after)
}

/// Not part of a path, address, file or identifier (`bitcoin/bitcoin`, `bitcoin-core`).
fn lower_prose_word(before: &str, after: &str) -> bool {
    let joined = |c: Option<char>| matches!(c, Some('/' | '.' | '-' | '@' | ':'));
    prose_word(before, after)
        && !joined(before.chars().next_back())
        && !joined(after.chars().next())
}

/// `sats` as the unit of an amount shown (`50,000 sats`, `${n} sats`, `<small>sats</small>`, "in
/// sats"), never as a name in code (`(sats) =>`, `"sats"`, `price_sats`).
fn amount_unit(before: &str, after: &str) -> bool {
    let line = before.rsplit('\n').next().unwrap_or(before);
    if starts_word(after) || ends_word(before) || line.contains("@param") {
        return false;
    }
    if before.ends_with("<small>") || before.ends_with(" in ") {
        return true;
    }
    let Some(spaced) = before.strip_suffix(' ') else {
        return false;
    };
    spaced
        .chars()
        .next_back()
        .is_some_and(|c| c.is_ascii_digit() || matches!(c, '}' | ')' | 'K' | 'M' | 'k'))
        && !after.trim_start().starts_with(['=', '('])
        && !after.strip_prefix('.').is_some_and(starts_word)
}

#[cfg(test)]
mod tests {
    use super::relabel;

    #[test]
    fn relabels_display_text_only() {
        for (from, to) in [
            ("Bitview Studio", "Dogeview Studio"),
            ("https://bitview.space/api", "https://dogeview.space/api"),
            (
                "<title>Explorer · Bitview</title>",
                "<title>Explorer · Dogeview</title>",
            ),
            ("import { BitviewClient }", "import { BitviewClient }"),
            (
                "/modules/bitview-client/index.js",
                "/modules/bitview-client/index.js",
            ),
            ("Bitcoin Core's selection", "Dogecoin Core's selection"),
            ("Bitcoin: [\"\", \" BTC\"]", "Bitcoin: [\"\", \" DOGE\"]"),
            ("series(v, \"In bitcoin\")", "series(v, \"In dogecoin\")"),
            (
                "usd: \"USD\", btc: \"BTC\", sats: \"sats\"",
                "usd: \"USD\", btc: \"DOGE\", sats: \"koinu\"",
            ),
            ("${d.bitcoin}", "${d.bitcoin}"),
            ("github.com/bitcoin/bitcoin", "github.com/bitcoin/bitcoin"),
            ("The BTC/USD price", "The DOGE/USD price"),
            ("BTC.com and BTC Guild", "BTC.com and BTC Guild"),
            ("/** @param {number} sats */", "/** @param {number} sats */"),
            ("SATS_PER_BTC", "SATS_PER_BTC"),
            ("0.01 BTC, 50,000 sats", "0.01 DOGE, 50,000 koinu"),
            ("`${grouped(v)} sats`", "`${grouped(v)} koinu`"),
            ("<small>sats</small>", "<small>koinu</small>"),
            ("1-10 Sats", "1-10 Koinu"),
            ("100K+ BTC", "100K+ DOGE"),
            (
                "const btc = (sats) => sats / 1e8",
                "const btc = (sats) => sats / 1e8",
            ),
            ("price_sats", "price_sats"),
            ("5 sat/vB and 2 sats/vB", "5 koinu/vB and 2 koinu/vB"),
            ("₿412K", "Ð412K"),
            ("The Satoshi Trail", "The Satoshi Trail"),
            (
                "fill=\"oklch(64.34% 0.191 44.41)\"",
                "fill=\"oklch(72.9% 0.132 94.8)\"",
            ),
            (
                "fill=%22oklch%2864.34%25 0.191 44.41%29%22",
                "fill=%22oklch%2872.9%25 0.132 94.8%29%22",
            ),
            (
                "<a href=\"https://bitcoinresearchkit.org\">Bitcoin Research Kit</a>",
                "<a href=\"https://dogeview.space/brk/\">Dogecoin Research Kit</a>",
            ),
            ("https://perf.bitview.dev", "https://perf.bitview.dev"),
            ("~/.bitcoin/bitcoin.conf", "~/.bitcoin/bitcoin.conf"),
        ] {
            assert_eq!(relabel(from), to, "{from}");
        }
    }

    #[test]
    fn states_dogecoin_facts() {
        for (from, to) in [
            (
                "order: spent outputs, plus the\ncoinbase outputs of blocks 91812 and 91722, which duplicates at 91842 and 91880 overwrote.",
                "order: spent outputs.",
            ),
            (
                "price series for each supported time period. Heights before 340,000 use\nbaked historical exchange prices; later heights use an on-chain oracle\nthat estimates price from round-USD transaction-output patterns. Zero\nbefore the first exchange price (block 74,641, August 2010).",
                "price series for each supported time period. Each block is priced at its day's DOGE/USD exchange close. Blocks before the first exchange close carry the earliest one.",
            ),
            (
                "Bitcoin's price in US dollars, block by block: from exchanges' history at first, then estimated from the chain itself.",
                "Dogecoin's price in US dollars, block by block: each at its day's exchange close.",
            ),
            (
                "Difficulty: how hard it is to find a block, adjusted every 2,016 blocks to keep them ten minutes apart.",
                "Difficulty: how hard it is to find a block, adjusted every block (DigiShield, since block 145,000) to keep them a minute apart.",
            ),
            (
                "Halving epoch: which subsidy era each block is in, one every 210,000 blocks.",
                "Halving epoch: which subsidy era each block is in, one every 100,000 blocks through 600,000, a fixed reward since.",
            ),
            (
                "Serialized block size in bytes, including witness data: its 80-byte header,\ntransaction-count CompactSize, and every serialized transaction.",
                "Serialized block size in bytes, as stored: its 80-byte header, the AuxPoW proof of a merge-mined block (from 371,337), the transaction-count CompactSize, and every serialized transaction.",
            ),
            (
                "Block size: the bytes each block takes, witness data included.",
                "Block size: the bytes each block takes, AuxPoW proof included.",
            ),
            (
                "Days until the next halving, at ten minutes a block.",
                "Days until the next halving, at a minute a block.",
            ),
            (
                "[GitHub](https://github.com/bitcoinresearchkit/mono/issues)",
                "[GitHub](https://github.com/DogecoinResearchKit/drk/issues)",
            ),
        ] {
            assert_eq!(relabel(from), to, "{from}");
        }
    }
}
