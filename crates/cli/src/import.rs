//! `spotify import`: finds export files in a directory or zip (including zips inside the
//! directory), groups them into exports by folder, and uploads the ones the server lacks.

use std::collections::{BTreeMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use chrono::{DateTime, Utc};
use cynic::{MutationBuilder, QueryBuilder};
use indicatif::{ProgressBar, ProgressStyle};
use spotify_export::FileKind;

use crate::client::Client;
use crate::graphql::{
    AccountQuery, AccountVariables, CreateExport, CreateExportInput, CreateExportVariables,
    FileExistsQuery, FileExistsVariables, FinalizeExport, FinalizeExportInput,
    FinalizeExportVariables, Upload, UploadExportFile, UploadExportFileInput,
    UploadExportFileVariables, UploadStatus,
};

enum Location {
    File(PathBuf),
    Zip { archive: PathBuf, entry: String },
}

impl Location {
    fn read(&self) -> anyhow::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        match self {
            Self::File(path) => {
                File::open(path)
                    .with_context(|| format!("opening {}", path.display()))?
                    .read_to_end(&mut bytes)?;
            }
            Self::Zip { archive, entry } => {
                let mut zip = zip::ZipArchive::new(File::open(archive)?)
                    .with_context(|| format!("opening {}", archive.display()))?;
                zip.by_name(entry)?.read_to_end(&mut bytes)?;
            }
        }
        Ok(bytes)
    }
}

struct Candidate {
    name: String,
    kind: FileKind,
    location: Location,
}

/// Files found under one folder (or one folder inside a zip), uploaded as a single export.
#[derive(Default)]
struct Group {
    files: Vec<Candidate>,
    unrecognized: usize,
}

enum Plan {
    Upload {
        sha256: String,
        bytes: Vec<u8>,
    },
    AlreadyImported,
    /// Same content as another file earlier in this run, e.g. a zip and its extracted copy.
    RepeatedInRun,
    NotImportable,
}

pub async fn run(client: &Client, account: &str, path: &Path, dry_run: bool) -> anyhow::Result<()> {
    let found = client
        .run(AccountQuery::build(AccountVariables {
            slug: account.to_owned(),
        }))
        .await?;
    if found.account.is_none() {
        bail!(
            "account {account:?} does not exist, create it with `spotify accounts add {account}`"
        );
    }

    let groups = discover(path)?;
    if groups.is_empty() {
        bail!("no Spotify export files found in {}", path.display());
    }

    let mut seen = HashSet::new();
    let mut totals = Totals::default();

    for (label, group) in groups {
        let mut planned = Vec::new();
        let mut as_of: Option<DateTime<Utc>> = None;

        for file in group.files {
            let bytes = file.location.read()?;
            let sha256 = hex::encode(<sha2::Sha256 as sha2::Digest>::digest(&bytes));

            if let Some(ts) = spotify_export::max_timestamp(file.kind, &bytes)
                .with_context(|| format!("reading {label}/{}", file.name))?
            {
                as_of = as_of.max(Some(ts));
            }

            let plan = if !file.kind.is_importable() {
                Plan::NotImportable
            } else if !seen.insert(sha256.clone()) {
                Plan::RepeatedInRun
            } else if client
                .run(FileExistsQuery::build(FileExistsVariables {
                    account: account.to_owned(),
                    sha256: sha256.clone(),
                }))
                .await?
                .file_exists
            {
                Plan::AlreadyImported
            } else {
                Plan::Upload { sha256, bytes }
            };

            planned.push((file.name, file.kind, plan));
        }

        let uploads = planned
            .iter()
            .filter(|(_, _, p)| matches!(p, Plan::Upload { .. }))
            .count();
        let as_of = as_of.unwrap_or_else(Utc::now);

        println!(
            "{label}  (as of {}, {} to upload, {} other files ignored)",
            as_of.format("%Y-%m-%d"),
            uploads,
            group.unrecognized
        );

        if dry_run || uploads == 0 {
            for (name, kind, plan) in &planned {
                let action = match plan {
                    Plan::Upload { .. } => "upload",
                    Plan::AlreadyImported => "already imported",
                    Plan::RepeatedInRun => "duplicate of another file",
                    Plan::NotImportable => "not importable",
                };
                println!("  {name:<50} {:<26} {action}", kind.as_str());
            }
            totals.skipped += planned.len() - uploads;
            continue;
        }

        let export = client
            .run(CreateExport::build(CreateExportVariables {
                input: CreateExportInput {
                    account: account.to_owned(),
                    label: label.clone(),
                    as_of,
                },
            }))
            .await?
            .create_export
            .export;

        let progress = ProgressBar::new(uploads as u64).with_style(
            ProgressStyle::with_template("  {bar:30} {pos}/{len} {msg}").expect("valid template"),
        );

        for (name, _, plan) in planned {
            let Plan::Upload { sha256, bytes } = plan else {
                totals.skipped += 1;
                continue;
            };

            progress.set_message(name.clone());
            let result = client
                .upload(
                    UploadExportFile::build(UploadExportFileVariables {
                        input: UploadExportFileInput {
                            export_id: export.id.clone(),
                            name: name.clone(),
                            sha256,
                        },
                        file: Upload,
                    }),
                    &name,
                    bytes,
                )
                .await
                .with_context(|| format!("uploading {label}/{name}"))?
                .upload_export_file;

            match (result.status, result.file) {
                (UploadStatus::Imported, Some(file)) => {
                    totals.files += 1;
                    totals.rows += file.rows_total as i64;
                    totals.inserted += file.rows_inserted as i64;
                    totals.merged += file.rows_merged as i64;
                    progress.println(format!(
                        "  {name:<50} {:>6} rows, {:>6} new, {:>6} merged",
                        file.rows_total, file.rows_inserted, file.rows_merged
                    ));
                }
                (status, _) => {
                    totals.skipped += 1;
                    progress.println(format!("  {name:<50} {status:?}"));
                }
            }
            progress.inc(1);
        }
        progress.finish_and_clear();

        client
            .run(FinalizeExport::build(FinalizeExportVariables {
                input: FinalizeExportInput {
                    export_id: export.id,
                },
            }))
            .await?;
    }

    println!(
        "\n{} files imported ({} rows: {} new plays, {} merged), {} skipped{}",
        totals.files,
        totals.rows,
        totals.inserted,
        totals.merged,
        totals.skipped,
        if dry_run { " (dry run)" } else { "" }
    );

    Ok(())
}

#[derive(Default)]
struct Totals {
    files: usize,
    rows: i64,
    inserted: i64,
    merged: i64,
    skipped: usize,
}

fn discover(root: &Path) -> anyhow::Result<BTreeMap<String, Group>> {
    let mut groups: BTreeMap<String, Group> = BTreeMap::new();
    let base = root.parent().unwrap_or(root);
    let label_of = |path: &Path| {
        path.strip_prefix(base)
            .unwrap_or(path)
            .display()
            .to_string()
    };

    let walker = walkdir::WalkDir::new(root).sort_by_file_name();
    for entry in walker {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy();

        if file_name.ends_with(".zip") {
            discover_zip(path, &label_of(path), &mut groups)?;
            continue;
        }

        let label = label_of(path.parent().unwrap_or(root));
        add(&mut groups, label, &file_name, || {
            Location::File(path.to_owned())
        });
    }

    Ok(groups)
}

fn discover_zip(
    path: &Path,
    label: &str,
    groups: &mut BTreeMap<String, Group>,
) -> anyhow::Result<()> {
    let zip = zip::ZipArchive::new(File::open(path)?)
        .with_context(|| format!("opening {}", path.display()))?;

    for entry in zip.file_names() {
        let entry = entry?;
        let entry = entry.as_ref();
        if entry.ends_with('/') || entry.starts_with("__MACOSX/") {
            continue;
        }

        let (dir, file_name) = entry.rsplit_once('/').unwrap_or(("", entry));
        let label = if dir.is_empty() {
            label.to_owned()
        } else {
            format!("{label}/{dir}")
        };

        add(groups, label, file_name, || Location::Zip {
            archive: path.to_owned(),
            entry: entry.to_owned(),
        });
    }

    Ok(())
}

fn add(
    groups: &mut BTreeMap<String, Group>,
    label: String,
    file_name: &str,
    location: impl FnOnce() -> Location,
) {
    if file_name.starts_with('.') {
        return;
    }

    let group = groups.entry(label).or_default();
    match FileKind::classify(file_name) {
        Some(kind) => group.files.push(Candidate {
            name: file_name.to_owned(),
            kind,
            location: location(),
        }),
        None => group.unrecognized += 1,
    }
}
