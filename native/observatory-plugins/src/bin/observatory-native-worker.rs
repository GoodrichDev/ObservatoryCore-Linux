use anyhow::{Context, Result};
use observatory_plugins::worker_rpc::{self, WorkerInfo, WorkerKind};
fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let mut kind = None;
    let mut hello = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--kind" => {
                kind = Some(
                    args.next()
                        .context("--kind needs a plugin name")?
                        .parse::<WorkerKind>()?,
                )
            }
            "--hello" => hello = true,
            _ => anyhow::bail!("Unknown native worker option {arg}"),
        }
    }
    let kind = kind.context("Expected --kind BioInsights|Evaluator|Hecate|StellarOverlay")?;
    if hello {
        println!("{}", serde_json::to_string(&WorkerInfo::current(kind))?);
        return Ok(());
    }
    worker_rpc::serve(kind, std::io::stdin().lock(), std::io::stdout().lock())
}
