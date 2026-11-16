//! Under `dogecoin`, the showcases to embed are Dogeview's: a copy of `showcases/` with its display
//! text relabeled, then `dogecoin/` laid over it for what text rules can't change (images).

use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

/// The extensions of the files relabeled; any other is copied as is.
const TEXT: &[&str] = &[
    "html",
    "js",
    "mjs",
    "json",
    "txt",
    "xml",
    "md",
    "svg",
    "webmanifest",
    "css",
];
/// Paths (from the showcases' root, by prefix) copied as is: the generated clients are code, and
/// the release notes are upstream's history, true of Bitcoin.
const VERBATIM: &[&str] = &["modules", "changes/CHANGELOG.md", "changes/v"];
/// The favicon's link, under a URL of its own, so browsers holding Bitview's ask for Dogeview's.
const FAVICON: (&str, &str) = ("favicon/favicon.svg\"", "favicon/favicon.svg?v=dogeview\"");
/// Bitcoin's chain facts and words written into the pages, and Dogecoin's: (path, upstream's
/// text, Dogecoin's), every occurrence replaced before relabeling. Each must still be found, so an
/// upstream rewrite fails the build rather than ship Bitcoin's numbers.
const FACTS: &[(&str, &str, &str)] = &[
    (
        "_chain/index.html",
        "const HALVING_EPOCH_BLOCKS = 210_000;",
        "const HALVING_EPOCH_BLOCKS = 100_000;",
    ),
    (
        "_chain/index.html",
        "const TARGET_BLOCK_SECONDS = 600;",
        "const TARGET_BLOCK_SECONDS = 60;",
    ),
    (
        "_chain/index.html",
        r#"TARGET_BLOCK_SECONDS, "~10mn")"#,
        r#"TARGET_BLOCK_SECONDS, "~1mn")"#,
    ),
    ("_chain/index.html", "initial-value: 600;", "initial-value: 60;"),
    (
        "_chain/index.html",
        "var(--block-interval, 600)",
        "var(--block-interval, 60)",
    ),
    ("calendar/index.html", "210_000", "100_000"),
    (
        "globe/index.html",
        r#"        ["2012-11-01", "1st halving"],
        ["2016-07-01", "2nd halving"],
        ["2020-05-01", "3rd halving"],
        ["2021-05-01", "China bans mining"],
        ["2024-04-01", "4th halving"],"#,
        r#"        ["2014-02-01", "1st halving"],
        ["2014-09-01", "AuxPoW: merged mining"],
        ["2015-02-01", "Fixed reward"],
        ["2021-05-01", "China bans mining"],"#,
    ),
    (
        "index.html",
        "market stands, day by day since 2011",
        "market stands, day by day since 2014",
    ),
    (
        "index.html",
        "live and since 2011.",
        "live and since 2014.",
    ),
    (
        "studio/index.html",
        "const KNOWN_HALVINGS = [1354116278, 1468082773, 1589225023, 1713571767], HALVING_BLOCKS = 210_000;
      /** The halvings mined so far: those known, and any since, placed from the tip's time back (a block each ten minutes). */
      const minedHalvings = () => [...KNOWN_HALVINGS, ...Array.from({ length: Math.max(0, Math.floor(liveHeight / HALVING_BLOCKS) - KNOWN_HALVINGS.length) }, (_, k) => (liveAt || Date.now() / 1000) - (liveHeight - (KNOWN_HALVINGS.length + k + 1) * HALVING_BLOCKS) * 600)];
      /** The halvings to come, the next `count`, estimated (a block each ten minutes from the tip): each its number and time. */
      const nextHalvings = (count = 3) => (liveHeight ? Array.from({ length: count }, (_, k) => ((n) => ({ n, time: Date.now() / 1000 + (n * HALVING_BLOCKS - liveHeight) * 600 }))(Math.floor(liveHeight / HALVING_BLOCKS) + 1 + k)) : []);",
        "const KNOWN_HALVINGS = [1392346781, 1398695540, 1405442852, 1412258966, 1418570422, 1424881308];
      /** The halvings mined: blocks 100,000 to 600,000, the last in February 2015. */
      const minedHalvings = () => KNOWN_HALVINGS;
      /** None to come: the reward has been a fixed 10,000 DOGE since block 600,000. */
      const nextHalvings = (count = 3) => [];",
    ),
    (
        "studio/index.html",
        "The halvings: each block a multiple of 210,000 cut the reward in half; these, as they were mined (210,000 to
       * 840,000). The next, a block every ten minutes from the tip's: an estimate, drawn as one.",
        "The halvings: each block a multiple of 100,000 through 600,000 cut the reward in half; these, as they were
       * mined. None come after: the reward has been a fixed 10,000 DOGE since.",
    ),
    (
        "studio/index.html",
        "halving: (210_000 * 600) / 86400",
        "halving: 4 * 365.25",
    ),
    (
        "studio/index.html",
        "not at the next block,\n        // ten minutes on;",
        "not at the next block,\n        // a minute on;",
    ),
    (
        "studio/index.html",
        "Math.floor(facts.height / 210_000)",
        "Math.floor(facts.height / 100_000)",
    ),
    (
        "studio/index.html",
        r#""Every 210,000 blocks""#,
        r#""Six, the last in 2015""#,
    ),
    (
        "studio/index.html",
        r#""halvings": true: a line at each halving, and at the next three, estimated (a block each ten minutes)."#,
        r#""halvings": true: a line at each of the six halvings (blocks 100,000 to 600,000, the last in February 2015)."#,
    ),
    (
        "studio/llms.txt",
        r#""halvings": true: a line at each halving, and at the next three, estimated (a block each ten minutes)."#,
        r#""halvings": true: a line at each of the six halvings (blocks 100,000 to 600,000, the last in February 2015)."#,
    ),
    // The `halving` span stays four years, the window Dogeview's halving-cycle series use.
    (
        "studio/index.html",
        "halving (210,000 blocks)",
        "halving (four years)",
    ),
    ("studio/llms.txt", "halving (210,000 blocks)", "halving (four years)"),
    (
        "studio/index.html",
        "a halving its 210,000 blocks at ten minutes each",
        "a halving four years",
    ),
    (
        "studio/index.html",
        "(3 January 2009): all of history's start",
        "(6 December 2013): all of history's start",
    ),
    (
        "studio/index.html",
        "const GENESIS = 1230940800;",
        "const GENESIS = 1386288000;",
    ),
    (
        "studio/index.html",
        r#"year: [2009, "y and at are years, 2009 (the genesis block's) and on"]"#,
        r#"year: [2013, "y and at are years, 2013 (the genesis block's) and on"]"#,
    ),
    (
        "studio/index.html",
        "1 from 2012, 2 from 2016, 3 from 2020, 4 from 2024",
        "1 to 5 through 2014 (blocks 100,000 to 500,000), 6 from February 2015 (block 600,000, the fixed reward)",
    ),
    (
        "studio/index.html",
        "(2009 the first, the genesis block's)",
        "(2013 the first, the genesis block's)",
    ),
    (
        "studio/index.html",
        "the price's start in 2010",
        "the price's start in 2013",
    ),
    (
        "studio/index.html",
        "(price before 2010 starts at zero)",
        "(the first exchange close is carried back to genesis)",
    ),
    (
        "studio/index.html",
        r#"txs: "transactions", utxos: "UTXOs" };"#,
        r#"txs: "transactions", utxos: "UTXOs", btc: "DOGE", sats: "koinu", sat: "koinu" };"#,
    ),
    (
        "studio/presets.js",
        "* 210_000 > facts.height",
        "* 100_000 > facts.height",
    ),
    (
        "studio/presets.js",
        "(facts?.height ?? 840_000) / 210_000",
        "Math.min(facts?.height ?? 600_000, 600_000) / 100_000",
    ),
    (
        "studio/presets.js",
        r#"label: `${n ? 2008 + 4 * n : 2009}–${n + 1 < cycles ? 2012 + 4 * n : "now"}`"#,
        r#"label: `${["Dec 2013", "Feb 2014", "Apr 2014", "Jul 2014", "Oct 2014", "Dec 2014", "Feb 2015"][n]}–${n + 1 < cycles ? ["Feb 2014", "Apr 2014", "Jul 2014", "Oct 2014", "Dec 2014", "Feb 2015"][n] : "now"}`"#,
    ),
    (
        "studio/presets.js",
        r#"levels: [{ value: 600, color: "gray", dash: 3 }]"#,
        r#"levels: [{ value: 60, color: "gray", dash: 3 }]"#,
    ),
    ("studio/presets.js", "(date - 2009-01-03)", "(date - 2013-12-06)"),
    (
        "series/index.html",
        r#"avg: "average", opreturn: "OP_RETURN" });"#,
        r#"avg: "average", opreturn: "OP_RETURN", btc: "DOGE", sats: "koinu", sat: "koinu" });"#,
    ),
    (
        "series/index.html",
        r#"["Value type", info.type]"#,
        r#"["Value type", ({ Bitcoin: "Dogecoin", Sats: "Koinu" })[info.type] ?? info.type]"#,
    ),
    // Dogecoin's dates: genesis on 6 December 2013; its prices start there, the first close
    // carried back.
    (
        "index.html",
        "Every day of every year since 2009,",
        "Every day of every year since 2013,",
    ),
    ("index.html", "what any plan since 2010", "what any plan since 2014"),
    ("stack/index.html", "what any plan since 2010", "what any plan since 2014"),
    (
        "calendar/index.html",
        "Every day of every year since 2009,",
        "Every day of every year since 2013,",
    ),
    (
        "calendar/index.html",
        r#"GENESIS = "2009-01-03";"#,
        r#"GENESIS = "2013-12-06";"#,
    ),
    ("calendar/index.html", "y <= 2009;", "y <= 2013;"),
    ("calendar/index.html", "&& y > 2009,", "&& y > 2013,"),
    (
        "calendar/index.html",
        "for (let i = 0; i < 144; i++)",
        "for (let i = 0; i < 1440; i++)",
    ),
    (
        "compass/index.html",
        "/** The first day read: the price begins in July 2010, nothing before it counts. */\n      const START = (Date.UTC(2010, 6, 1) - EPOCH) / DAY;",
        "/** The first day read: the price begins in February 2014, its days before the first close's carried. */\n      const START = (Date.UTC(2014, 1, 1) - EPOCH) / DAY;",
    ),
    // No on-chain oracle on Dogecoin, and the Trail and Logs stay served, off the home page.
    (
        "index.html",
        "      ]);\n      // (The developers' named",
        "      ].filter((t) => ![\"oracle\", \"trail\", \"logs\"].includes(t.id)));\n      // (The developers' named",
    ),
    (
        "index.html",
        r#"["studio", "oracle", "globe", "trail", "stack", "calendar"]"#,
        r#"["studio", "globe", "stack", "calendar"]"#,
    ),
    // perf.bitview.dev benchmarks Bitcoin; Dogeview hasn't been measured yet.
    (
        "index.html",
        r##"<a href="https://perf.bitview.dev">Benchmarks<svg class="glyph" aria-hidden="true"><use href="#i-out" /></svg></a>"##,
        "",
    ),
    (
        "dev/index.html",
        "        <a href=\"https://perf.bitview.dev\">Benchmarks</a>\n",
        "",
    ),
    (
        "dev/index.html",
        r#"        <dl class="specs" aria-label="Bitview, measured">
          <div><dt>First index</dt><dd>~1½ h<small>from block 0, 16 GB Mac mini</small></dd></div>
          <div><dt>Peak memory</dt><dd>8.4 GiB<small>16 GB recommended</small></dd></div>
          <div><dt>On disk</dt><dd>290 GiB<small>beside Bitcoin Core's</small></dd></div>
          <div><dt>Series</dt><dd>21,154<small>across their indexes</small></dd></div>
          <div><dt>API operations</dt><dd>98<small>series, blocks, mempool…</small></dd></div>
        </dl>
        <p class="specs-foot">Measured on an Apple M6 Mac mini, 16 GB, external NVMe SSD; every run at <a href="https://perf.bitview.dev">perf.bitview.dev</a>.</p>"#,
        r#"        <p class="specs-foot">Not yet benchmarked on Dogecoin.</p>"#,
    ),
    (
        "dev/index.html",
        r#"<span class="label">Install, or update</span><button type="button" data-copy='rustup update &amp;&amp; RUSTFLAGS="-C target-cpu=native" cargo install --locked bitviewd'>Copy</button></div>
          <pre class="terminal"><span class="line"><span class="prompt">$</span> rustup update &amp;&amp; RUSTFLAGS="-C <span class="w">target-cpu=native</span>" cargo install <span class="w">--locked</span> bitviewd</span>"#,
        r#"<span class="label">Install</span><button type="button" data-copy='git clone https://github.com/IndigoNakamoto/drk &amp;&amp; cd drk &amp;&amp; RUSTFLAGS="-C target-cpu=native" cargo install --locked --path crates/bitviewd --features dogecoin'>Copy</button></div>
          <pre class="terminal"><span class="line"><span class="prompt">$</span> git clone https://github.com/IndigoNakamoto/drk &amp;&amp; cd drk</span><span class="line"><span class="prompt">$</span> RUSTFLAGS="-C <span class="w">target-cpu=native</span>" cargo install <span class="w">--locked</span> --path crates/bitviewd <span class="w">--features dogecoin</span></span>"#,
    ),
    (
        "dev/index.html",
        "16 GB of memory recommended, and about 290 GiB of disk on top of Bitcoin Core's.",
        "16 GB of memory recommended; its disk use on Dogecoin isn't measured yet.",
    ),
    (
        "dev/index.html",
        "About an hour and a half on a 16 GB Mac mini with a fast SSD;",
        "Not measured on Dogecoin yet;",
    ),
    (
        "index.html",
        r#"copy: "cargo install --locked bitviewd" }"#,
        r#"copy: "git clone https://github.com/IndigoNakamoto/drk && cd drk && cargo install --locked --path crates/bitviewd --features dogecoin" }"#,
    ),
    (
        "index.html",
        r#"" cargo install", el("br"), "--locked bitviewd")"#,
        r#"" cargo install", el("br"), "--features dogecoin")"#,
    ),
    // Sub-cent prices: DOGE traded at hundredths of a cent, so no figure rounds to whole cents or
    // dollars.
    (
        "index.html",
        r#"(v >= 1e6 ? `$${short(v)}` : `$${grouped(v)}`)"#,
        r#"(v >= 1e6 || v < 100 ? `$${short(v)}` : `$${grouped(v)}`)"#,
    ),
    (
        "compass/index.html",
        r#"(n < 10 ? `$${n.toFixed(2)}` : `$${grouped(n)}`)"#,
        r#"(n < 10 ? `$${Number(n.toPrecision(3))}` : `$${grouped(n)}`)"#,
    ),
    (
        "studio/index.html",
        r#"const toTheCent = new Intl.NumberFormat("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 });"#,
        r#"const toTheCent = new Intl.NumberFormat("en-US", { minimumSignificantDigits: 2, maximumSignificantDigits: 4 });"#,
    ),
    (
        "trail/index.html",
        "return `$${Math.round(value)}`;",
        "return `$${Number(value.toPrecision(3))}`;",
    ),
    // The live price is the latest exchange close, not read from the chain.
    (
        "index.html",
        "</b> read from the chain just now",
        "</b>, the latest exchange close",
    ),
    (
        "studio/index.html",
        "`Price ${dollarsAndCents(livePrice)}, as it trades`",
        "`Price ${dollarsAndCents(livePrice)}, at the latest close`",
    ),
    // OpenSats supports upstream's project, not this fork.
    (
        "dev/index.html",
        "            <a class=\"button\" href=\"https://opensats.org/projects/bitcoinresearchkit\">Supported by OpenSats ↗</a>\n",
        "",
    ),
    (
        "brk/index.html",
        "            <a class=\"button\" href=\"https://opensats.org/projects/bitcoinresearchkit\">Supported by OpenSats ↗</a>\n",
        "",
    ),
];

fn main() -> io::Result<()> {
    println!("cargo::rerun-if-env-changed=CARGO_FEATURE_DOGECOIN");
    if env::var_os("CARGO_FEATURE_DOGECOIN").is_none() {
        return Ok(());
    }
    println!("cargo::rerun-if-changed=showcases");
    println!("cargo::rerun-if-changed=dogecoin");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("showcases");
    if out.exists() {
        fs::remove_dir_all(&out)?;
    }
    for (file, ..) in FACTS {
        assert!(
            Path::new("showcases").join(file).exists(),
            "no showcases/{file}"
        );
    }
    copy(Path::new("showcases"), &out, "", true)?;
    let overlay = Path::new("dogecoin");
    if overlay.exists() {
        copy(overlay, &out, "", false)?;
    }
    Ok(())
}

fn copy(from: &Path, to: &Path, relative: &str, relabel: bool) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let path = entry?.path();
        let name = path.file_name().unwrap();
        let target = to.join(name);
        let relative = format!("{relative}{}", name.to_string_lossy());
        let relabel = relabel && !VERBATIM.iter().any(|v| relative.starts_with(v));
        if path.is_dir() {
            copy(&path, &target, &format!("{relative}/"), relabel)?;
        } else if relabel
            && path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| TEXT.contains(&e))
        {
            let mut text = fs::read_to_string(&path)?;
            for (_, from, to) in FACTS.iter().filter(|(file, ..)| *file == relative) {
                assert!(text.contains(from), "{relative} no longer has {from:?}");
                text = text.replace(from, to);
            }
            let text = text.replace(FAVICON.0, FAVICON.1);
            fs::write(&target, brk_relabel::relabel(&text))?;
        } else {
            fs::copy(&path, &target)?;
        }
    }
    Ok(())
}
