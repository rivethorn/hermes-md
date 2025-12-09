use std::{fs, path::Path};

use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use reqwest::multipart;
use serde::Deserialize;
use slug::slugify;

#[derive(Parser)]
#[command(name = "supamarker")]
#[command(about = "Publish markdown posts to Supabase (storage + posts table)")]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Publish a local markdown file
    Publish { path: String },
    /// Delete a post by slug
    Delete { slug: String },
}

#[derive(Debug, Deserialize)]
struct FrontMatter {
    title: String,
    summary: Option<String>,
    tags: Option<Vec<String>>,
    slug: Option<String>,
}

async fn publish(
    supabase_url: &str,
    service_key: &str,
    bucket: &str,
    table: &str,
    path: &str,
) -> Result<()> {
    // 1) Read file
    let md = fs::read_to_string(path).with_context(|| format!("reading {}", path))?;

    // 2) Extract frontmatter (simple YAML between --- markers)
    //    We'll try to find `---\n...yaml...\n---\n` at start
    let (fm_opt, _) = parse_frontmatter(&md)?;
    let fm = fm_opt
        .ok_or_else(|| anyhow!("Frontmatter not found or invalid. Provide YAML frontmatter."))?;

    // 3) Slug
    let slug = fm.slug.clone().unwrap_or_else(|| {
        Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| slugify(s))
            .unwrap_or_else(|| slugify(&fm.title))
    });

    // 4) Upload markdown file to Supabase Storage via REST API
    // Endpoint: POST {SUPABASE_URL}/storage/v1/object/{bucket}/{object}
    // (multipart/form-data field "file")
    let upload_url = format!(
        "{}/storage/v1/object/{}/{}.md",
        supabase_url.trim_end_matches('/'),
        bucket,
        slug
    );

    let client = reqwest::Client::new();

    let file_name = format!("{}.md", slug);
    let part = multipart::Part::text(md.clone())
        .file_name(file_name)
        .mime_str("text/markdown")?;
    // note: we use "file" field like the JS SDK/multipart examples do
    let form = multipart::Form::new().part("file", part);

    let upload_resp = client
        .post(&upload_url)
        .header(AUTHORIZATION, format!("Bearer {}", service_key))
        // recommended Accept header
        .header(ACCEPT, "application/json")
        .multipart(form)
        .send()
        .await
        .with_context(|| "uploading markdown to Supabase Storage")?;

    if !upload_resp.status().is_success() {
        let status = upload_resp.status();
        let text = upload_resp.text().await.unwrap_or_default();
        return Err(anyhow!("Storage upload failed: {} - {}", status, text));
    }

    println!("✓ uploaded markdown to storage as {}/{}.md", bucket, slug);

    // 5) Upsert metadata into your table via PostgREST (Supabase REST)
    // Use the PostgREST endpoint: {SUPABASE_URL}/rest/v1/{SUPABASE_TABLE}
    // We'll POST and set "Prefer: resolution=merge-duplicates" so conflict = upsert (merge)
    let rest_url = format!("{}/rest/v1/{}", supabase_url.trim_end_matches('/'), table);

    // Build JSON payload (we send an array with a single row)
    let payload = serde_json::json!([{
        "slug": slug,
        "title": fm.title,
        "summary": fm.summary.unwrap_or_default(),
        "tags": fm.tags.unwrap_or_default()
    }]);

    let metadata_resp = client
        .post(&rest_url)
        .header(AUTHORIZATION, format!("Bearer {}", service_key))
        // required by Supabase PostgREST to identify project and allow the key
        .header("apikey", service_key)
        // ask PostgREST to merge duplicates (upsert)
        .header("Prefer", "resolution=merge-duplicates")
        .header(CONTENT_TYPE, "application/json")
        .json(&payload)
        .send()
        .await
        .with_context(|| format!("inserting/upserting metadata into {} table", table))?;

    if !metadata_resp.status().is_success() {
        let status = metadata_resp.status();
        let text = metadata_resp.text().await.unwrap_or_default();
        return Err(anyhow!("DB upsert failed: {} - {}", status, text));
    }

    println!(
        "✓ upserted metadata into {} table for slug `{}`",
        table, slug
    );
    println!("Published ✅: {}", fm.title);

    Ok(())
}

async fn delete_post(
    supabase_url: &str,
    service_key: &str,
    bucket: &str,
    slug: &str,
    table: &str,
) -> Result<()> {
    let client = reqwest::Client::new();

    // 1) Delete markdown from storage
    let storage_url = format!(
        "{}/storage/v1/object/{}/{}",
        supabase_url.trim_end_matches('/'),
        bucket,
        slug
    );

    let storage_resp = client
        .delete(&storage_url)
        .header(AUTHORIZATION, format!("Bearer {}", service_key))
        .header("apikey", service_key) // needed for service role
        .header("Accept", "application/json")
        .send()
        .await?;

    if !storage_resp.status().is_success() {
        let status = storage_resp.status();
        let text = storage_resp.text().await.unwrap_or_default();
        return Err(anyhow!(
            "Failed to delete storage file: {} - {}",
            status,
            text
        ));
    }

    println!("✓ Deleted markdown from storage: {}/{}", bucket, slug);

    // 2) Delete metadata from DB
    let rest_url = format!(
        "{}/rest/v1/{}?slug=eq.{}",
        supabase_url.trim_end_matches('/'),
        table,
        slug.replace(".md", "")
    );

    let db_resp = client
        .delete(&rest_url)
        .header(AUTHORIZATION, format!("Bearer {}", service_key))
        .header("apikey", service_key)
        .header("Accept", "application/json")
        .send()
        .await?;

    if !db_resp.status().is_success() {
        let status = db_resp.status();
        let text = db_resp.text().await.unwrap_or_default();
        return Err(anyhow!(
            "Failed to delete metadata from DB: {} - {}",
            status,
            text
        ));
    }

    println!(
        "✓ Deleted metadata from {} table for slug `{}`",
        table, slug
    );
    println!("Post `{}` deleted successfully ✅", slug);

    Ok(())
}

/// Very small frontmatter parser: returns (Option<FrontMatter>, content_without_fm)
fn parse_frontmatter(s: &str) -> Result<(Option<FrontMatter>, String)> {
    let s = s.trim_start();
    if !s.starts_with("---") {
        return Ok((None, s.to_string()));
    }

    // find second '---' marker
    let mut parts = s.splitn(3, "---");
    // first split gives empty before first '---'
    let _ = parts.next();
    let yaml = parts
        .next()
        .ok_or_else(|| anyhow!("no closing frontmatter marker"))?;
    let rest = parts.next().unwrap_or("");

    let yaml = yaml.trim();
    let rest = rest.trim_start_matches('\n').to_string();

    let fm: FrontMatter = serde_yaml::from_str(yaml).context("parsing YAML frontmatter")?;
    Ok((Some(fm), rest))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let args = Cli::parse();

    // Load env from environment variables (set these in your shell)
    let supabase_url = std::env::var("SUPABASE_URL")
        .context("SUPABASE_URL must be set (e.g. https://xxxxx.supabase.co)")?;
    let service_key = std::env::var("SUPABASE_SERVICE_KEY")
        .context("SUPABASE_SERVICE_KEY must be set (service role key for CLI)")?;
    let bucket = std::env::var("SUPABASE_BUCKET").unwrap_or_else(|_| "blog".to_string());
    let table = std::env::var("SUPABASE_TABLE").unwrap_or_else(|_| "posts".to_string());

    match args.cmd {
        Commands::Publish { path } => {
            publish(&supabase_url, &service_key, &bucket, &table, &path).await?;
        }
        Commands::Delete { slug } => {
            delete_post(&supabase_url, &service_key, &bucket, &slug, &table).await?;
        }
    }

    Ok(())
}
