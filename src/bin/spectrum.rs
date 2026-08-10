use anyhow::Result;
use clap::{Parser, Subcommand};
use spectrum::collect::mcp::measure;
use spectrum::hostile_mcp::{tool_list, Scenario};
use std::process::ExitCode;
use std::time::Instant;

#[derive(Parser)]
#[command(
    name = "spectrum",
    version,
    about = "A composition commitment for agent systems"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Measure the local cost of re-measuring an MCP tool list.
    BenchMcp {
        #[arg(long, default_value_t = 10_000)]
        iterations: u32,
        /// Tools in the synthetic list, so cost against list size is visible.
        #[arg(long, default_value_t = 8)]
        tools: u32,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}

fn synthetic(tools: u32) -> String {
    let items: Vec<String> = (0..tools)
        .map(|i| format!(r#"{{"name":"tool_{i}","parameters":{{"a":"number","b":"string"}}}}"#))
        .collect();
    format!(r#"{{"tools":[{}]}}"#, items.join(","))
}

fn run() -> Result<ExitCode> {
    match Cli::parse().cmd {
        Cmd::BenchMcp { iterations, tools } => {
            if iterations == 0 {
                anyhow::bail!("--iterations must be greater than 0");
            }

            // Warm up, so the first allocation does not land in the number.
            let body = synthetic(tools);
            for _ in 0..100 {
                measure("mcp://bench", &body)?;
            }

            let start = Instant::now();
            for _ in 0..iterations {
                measure("mcp://bench", &body)?;
            }
            let total = start.elapsed();
            let per = total / iterations;

            println!("tools           {tools}");
            println!("iterations      {iterations}");
            println!("per measurement {per:?}");
            println!(
                "as a share of the 15 ms per-action budget: {:.4}%",
                per.as_secs_f64() / 0.015 * 100.0
            );
            println!();
            println!(
                "LOCAL ONLY. This is canonicalization and hashing. It excludes the round trip \
                 to a remote MCP server, which is the operator's network and is not measured \
                 here. Do not report this as an end-to-end re-measurement cost."
            );

            // Sanity: the hostile server's own lists must measure too.
            for s in [Scenario::Stable, Scenario::RugPull, Scenario::AddedTool] {
                measure("mcp://hostile", &tool_list(s, 1))?;
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}
