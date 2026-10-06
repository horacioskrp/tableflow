//! Command-line interface for [`tableflow`]: read a form version and its
//! submissions (both JSON), write a tabular export.
//!
//! ```text
//! tableflow --format report survey.json submissions.json
//! tableflow --format xlsx -o out.xlsx survey.json submissions.json
//! ```

use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use serde_json::Value;
use tableflow::{Layout, MultipleSelect};

/// Export XLSForm submissions to CSV, XLSX, GeoJSON or a per-field report.
#[derive(Parser)]
#[command(name = "tableflow", version, about)]
struct Cli {
    /// Form version schema (JSON: `{ "content": { "survey": [...], ... } }`).
    schema: PathBuf,
    /// Submissions (a JSON array of objects).
    submissions: PathBuf,

    /// Output format.
    #[arg(long, value_enum, default_value_t = Format::Csv)]
    format: Format,
    /// Language for labels (field names are used when omitted).
    #[arg(long)]
    lang: Option<String>,
    /// `select_multiple` expansion: `both`, `summary` or `details`.
    #[arg(long, default_value = "both")]
    multiple_select: String,
    /// Title of the main table / GeoJSON collection.
    #[arg(long, default_value = "submissions")]
    title: String,
    /// Separator between hierarchy levels and expansion columns.
    #[arg(long, default_value = "/")]
    group_sep: String,
    /// Prefix each header with its enclosing groups' labels.
    #[arg(long)]
    hierarchy: bool,
    /// Extra submission key to append as a trailing column (repeatable).
    #[arg(long = "copy-field", value_name = "NAME")]
    copy_fields: Vec<String>,
    /// Tag column to emit as a header row, e.g. `hxl` (repeatable).
    #[arg(long = "tag-col", value_name = "NAME")]
    tag_cols: Vec<String>,
    /// Keep only this field (repeatable); all fields are kept when unset.
    #[arg(long = "filter-field", value_name = "NAME")]
    filter_fields: Vec<String>,
    /// Append a `<name>_URL` column after each media field.
    #[arg(long)]
    media_url: bool,
    /// Add an `_index` column to every section.
    #[arg(long)]
    force_index: bool,
    /// Disaggregate the report by this field (report format only).
    #[arg(long, value_name = "FIELD")]
    split_by: Option<String>,
    /// Write to this file instead of standard output.
    #[arg(long, short)]
    output: Option<PathBuf>,
}

/// Output formats the CLI can produce.
#[derive(Clone, Copy, ValueEnum)]
enum Format {
    /// Main section as CSV.
    Csv,
    /// Every section (main + repeats) as framed multi-section text.
    Tables,
    /// `.xlsx` workbook (one worksheet per table).
    Xlsx,
    /// GeoJSON `FeatureCollection`.
    Geojson,
    /// Per-field summary report (JSON).
    Report,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let version: Value = read_json(&cli.schema).context("reading the version schema")?;
    let submissions = match read_json(&cli.submissions).context("reading the submissions")? {
        Value::Array(items) => items,
        _ => anyhow::bail!("submissions file must be a JSON array of objects"),
    };

    let copy_fields: Vec<&str> = cli.copy_fields.iter().map(String::as_str).collect();
    let tag_cols: Vec<&str> = cli.tag_cols.iter().map(String::as_str).collect();
    let filter: Option<Vec<&str>> = (!cli.filter_fields.is_empty())
        .then(|| cli.filter_fields.iter().map(String::as_str).collect());
    let layout = Layout {
        lang: cli.lang.as_deref(),
        multiple_select: MultipleSelect::parse(&cli.multiple_select),
        group_sep: &cli.group_sep,
        hierarchy_in_labels: cli.hierarchy,
        copy_fields: &copy_fields,
        tag_cols: &tag_cols,
        filter_fields: filter.as_deref(),
        include_media_url: cli.media_url,
        force_index: cli.force_index,
    };

    let bytes = match cli.format {
        Format::Csv => tableflow::export_csv(&version, &submissions, &layout).into_bytes(),
        Format::Tables => {
            tableflow::export_tables_text(&version, &submissions, &cli.title, &layout).into_bytes()
        }
        Format::Xlsx => tableflow::export_xlsx(&version, &submissions, &cli.title, &layout)
            .context("building the XLSX workbook")?,
        Format::Geojson => {
            tableflow::export_geojson(&version, &submissions, cli.lang.as_deref(), &cli.title)
                .into_bytes()
        }
        Format::Report => tableflow::export_report(
            &version,
            &submissions,
            cli.lang.as_deref(),
            cli.split_by.as_deref(),
        )
        .into_bytes(),
    };

    match cli.output {
        Some(path) => {
            fs::write(&path, bytes).with_context(|| format!("writing {}", path.display()))?
        }
        None => io::stdout()
            .write_all(&bytes)
            .context("writing to stdout")?,
    }
    Ok(())
}

/// Read and parse a JSON file.
fn read_json(path: &PathBuf) -> Result<Value> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parsing JSON in {}", path.display()))
}
