use anyhow::Result;
use clap::{Parser, Subcommand};
use spectrum::collect::mcp::measure;
use spectrum::hostile_mcp::{tool_list, Scenario};
use std::path::PathBuf;
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
    /// Project a CycloneDX document under a profile and print the manifest digest.
    Compose {
        #[arg(long)]
        profile: PathBuf,
        #[arg(long)]
        bom: PathBuf,
    },
    /// Print what this profile does not capture.
    Coverage {
        #[arg(long)]
        profile: PathBuf,
    },
    /// Run one attack against every profile and report what each catches.
    Detect {
        #[arg(long)]
        scenario: String,
    },
    /// Serve one hostile-MCP scenario over HTTP, for manual exploration.
    ServeHostile {
        #[arg(long)]
        scenario: String,
        #[arg(long, default_value = "127.0.0.1:0")]
        addr: String,
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
        Cmd::Compose { profile, bom } => {
            let p = spectrum::profile::Profile::load(&profile)?;
            let text = std::fs::read_to_string(&bom)?;
            let components = spectrum::project::from_cyclonedx(&text, &p)?;
            let m = spectrum::manifest::Manifest::new(&p, components);
            println!("profile   {}", p.id);
            println!("components {}", m.components.len());
            println!("digest    {}", m.digest()?);
            println!();
            println!(
                "This digest is a commitment under the `{}` theory of what composition",
                p.id
            );
            println!("means. It is not comparable with a digest taken under another profile.");
            println!(
                "Run `spectrum coverage --profile {}` for what it does not capture.",
                profile.display()
            );
            Ok(ExitCode::SUCCESS)
        }
        Cmd::Coverage { profile } => {
            let p = spectrum::profile::Profile::load(&profile)?;
            println!("profile {}", p.id);
            println!();
            println!("This profile does NOT capture:");
            for c in &p.coverage {
                println!("  - {c}");
            }
            println!();
            if p.id == "runtime-with-context" {
                println!(
                    "PRIVACY: this profile digests retrieved content. A content-derived chunk \
                     digest is a cross-session, cross-agent fingerprint of what was read — two \
                     records carrying the same chunk digest prove both agents read the same \
                     document. OPEN FORK 7 is unresolved."
                );
                println!();
            }
            println!(
                "No profile captures which retrieved content influenced a decision. C1.2.2 asks \
                 for that link and nobody can compute it; this tool records presence and never \
                 names it influence."
            );
            Ok(ExitCode::SUCCESS)
        }
        Cmd::Detect { scenario } => {
            let attack = match scenario.as_str() {
                "none" => spectrum::detect::Attack::NoAttack,
                "tool-rug-pull" => spectrum::detect::Attack::ToolRugPull,
                "added-tool" => spectrum::detect::Attack::AddedTool,
                "content-poisoning" => spectrum::detect::Attack::ContentPoisoning,
                other => anyhow::bail!("unknown scenario `{other}`"),
            };
            let mut any = false;
            for name in ["build-manifest", "runtime-tools", "runtime-with-context"] {
                let p = spectrum::profile::Profile::load(
                    &std::path::Path::new("profiles").join(format!("{name}.toml")),
                )?;
                match spectrum::detect::run(attack, &p)? {
                    spectrum::detect::Outcome::Detected { component } => {
                        any = true;
                        println!("{name:<22} DETECTED   {component}");
                    }
                    spectrum::detect::Outcome::NotDetected => {
                        println!("{name:<22} not detected");
                    }
                }
            }
            Ok(if any {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            })
        }
        Cmd::ServeHostile { scenario, addr } => {
            let s = match scenario.as_str() {
                "stable" => Scenario::Stable,
                "rug-pull" => Scenario::RugPull,
                "added-tool" => Scenario::AddedTool,
                other => anyhow::bail!("unknown scenario `{other}`"),
            };
            spectrum::hostile_mcp::serve(&addr, s)?;
            Ok(ExitCode::SUCCESS)
        }
    }
}
