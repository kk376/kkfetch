#![warn(clippy::undocumented_unsafe_blocks)]
use clap::Parser;
use kkfetch::cli::Cli;
use kkfetch::context::FetchContext;
use kkfetch::modules::{ModuleId, ModuleRegistry};
use kkfetch::output::formatter::{render_json, render_layout, render_timings_grid};
use kkfetch::output::logo::match_logo;

fn main() {
    let cli = Cli::parse();

    // Version query
    if cli.version {
        println!("kkfetch {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    // Diagnostic check for installation health and conflicting binaries
    if cli.doctor {
        println!("kkfetch doctor: checking system installation");
        kkfetch::doctor::check_binary_shadowing(true);
        return;
    }

    // Early exit for shell completion scripts or discovery tooling
    if cli.list_modules {
        for module in ModuleId::all() {
            println!("{}", module.as_str());
        }
        return;
    }

    // Initialize execution context once to share terminal dimensions and OS release metadata
    let ctx = FetchContext::new(&cli);
    let registry = ModuleRegistry::new();
    let total_start = std::time::Instant::now();
    let (outputs, timings) = registry.collect_all_timed(&ctx);
    let total_elapsed = total_start.elapsed();

    // JSON export mode skips ANSI styling and ASCII logo formatting entirely
    if cli.json {
        println!("{}", render_json(&outputs));
        if cli.timings {
            eprintln!(
                "{}",
                render_timings_grid(&timings, total_elapsed, ctx.term_width, false)
            );
        }
        return;
    }

    // Resolve distro ASCII art using explicit CLI override, distro ID, or ID_LIKE fallback
    let logo = if ctx.no_logo {
        None
    } else {
        match_logo(
            ctx.logo_override.as_deref(),
            &ctx.os_info.distro_id,
            &ctx.os_info.distro_like,
        )
    };

    // Format side-by-side or stacked layout depending on terminal column width
    let rendered = render_layout(logo, &outputs, ctx.term_width, ctx.enable_color);
    if !rendered.is_empty() {
        println!("{}", rendered);
    }

    if cli.timings {
        println!(
            "{}",
            render_timings_grid(&timings, total_elapsed, ctx.term_width, ctx.enable_color)
        );
    }
}
