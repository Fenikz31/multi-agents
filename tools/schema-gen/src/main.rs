use clap::Parser;
use config_model::{json_schema_project, json_schema_providers};

#[derive(Parser, Debug)]
struct Args {
    /// Output directory for JSON Schemas
    #[arg(long, default_value = "docs/specs/schemas")] 
    out_dir: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    std::fs::create_dir_all(&args.out_dir)?;

    let proj = json_schema_project();
    let prov = json_schema_providers();

    std::fs::write(
        format!("{}/project.schema.json", args.out_dir),
        serde_json::to_vec_pretty(&proj)?,
    )?;
    std::fs::write(
        format!("{}/providers.schema.json", args.out_dir),
        serde_json::to_vec_pretty(&prov)?,
    )?;

    println!("Schemas written to {}", args.out_dir);
    Ok(())
}
