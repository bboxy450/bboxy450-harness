// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

#![allow(clippy::disallowed_methods)] // Startup, headless, and post-event-loop join only.
fn main() {
    if let Some(code) = bboxy450_harness::dream::c1::early_worker_entry() {
        std::process::exit(code);
    }
    ordinary_main();
}
// Keep the large ordinary CLI future/GUI stack frame behind early C1 admission.
// This is an entry boundary, not an OS/loader response-time guarantee.
#[inline(never)]
fn ordinary_main() {
    let code = if std::env::args().nth(1).as_deref() == Some("gui") {
        gui()
    } else {
        bbt_rt::handle().block_on(bboxy450_harness::frontends::headless::cli())
    };
    std::process::exit(code);
}
fn gui() -> i32 {
    if std::env::args().skip(2).any(|a| a == "--help" || a == "-h") {
        println!(
            "bboxy450-harness gui [--profile P] [--cwd D] [--resume SESSION]\nbboxy450-harness gui --replay FIXTURE.jsonl [--headless --screenshot PATH | --capture WxH [--no-chrome] [--zoom FACTOR (0.9–1.6, step 0.1)] [--expand all] [--open-menu profiles|slash] --screenshot PATH]"
        );
        return 0;
    }
    #[cfg(feature = "standalone")]
    {
        use bboxy450_harness::frontends::gui::{options::Options, smoke, window};
        let result = (|| {
            let options = Options::parse(std::env::args().skip(2))?;
            if options.capture.is_some() {
                return bbt_rt::handle().block_on(smoke::capture(options));
            }
            if options.headless {
                return bbt_rt::handle()
                    .block_on(smoke::run(options.replay.unwrap(), options.screenshot));
            }
            // The event loop is finished before this post-detach join. There are
            // no UI callbacks left to stall while admitted turns finish/fsync.
            if let Some(job) = window::run(options)? {
                bbt_rt::handle().block_on(job).map_err(|e| e.to_string())?;
            }
            Ok(())
        })();
        if let Err(error) = result {
            eprintln!("bboxy450-harness gui: {error}");
            return 1;
        }
        0
    }
    #[cfg(not(feature = "standalone"))]
    {
        eprintln!("bboxy450-harness gui requires --features standalone");
        1
    }
}
