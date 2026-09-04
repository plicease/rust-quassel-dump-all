mod db;
mod format;
mod sanitize;

use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Result, anyhow, bail};
use clap::{Parser, ValueEnum};

use db::{PgConfig, PostgresDb, QuasselDb, SqliteDb, buffer_type};
use format::{Event, HtmlRenderer, Renderer, TextRenderer, classify};
use sanitize::{sanitize_component, unique_filename};

#[derive(Copy, Clone, Eq, PartialEq, ValueEnum)]
enum OutputFormat {
    Text,
    Html,
}

/// Dump Quassel IRC logs for one user into per-network, per-channel files.
#[derive(Parser)]
#[command(name = "quassel-dump-all", version, about)]
struct Cli {
    /// Quassel username whose logs will be dumped
    #[arg(short = 'u', long)]
    user: String,

    /// Output directory (a subdirectory is created per network)
    #[arg(short = 'o', long, default_value = "quassel-dump-all-out")]
    out: PathBuf,

    /// Output file format
    #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,

    /// Only dump this network
    #[arg(short = 'n', long)]
    network: Option<String>,

    /// Only dump this channel/buffer (implies --network)
    #[arg(short = 'c', long)]
    channel: Option<String>,

    /// Path to a Quassel sqlite3 database file
    #[arg(long, value_name = "PATH")]
    sqlite: Option<PathBuf>,

    /// Connect to a PostgreSQL database instead of sqlite
    #[arg(long)]
    postgres: bool,

    /// PostgreSQL host
    #[arg(long, default_value = "localhost", requires = "postgres")]
    pg_host: String,

    /// PostgreSQL port
    #[arg(long, default_value_t = 5432, requires = "postgres")]
    pg_port: u16,

    /// PostgreSQL user
    #[arg(long, requires = "postgres")]
    pg_user: Option<String>,

    /// PostgreSQL password (falls back to PGPASSWORD env var, then an interactive prompt)
    #[arg(long, requires = "postgres")]
    pg_password: Option<String>,

    /// PostgreSQL database name
    #[arg(long, requires = "postgres")]
    pg_dbname: Option<String>,
}

fn open_db(cli: &Cli) -> Result<Box<dyn QuasselDb>> {
    match (&cli.sqlite, cli.postgres) {
        (Some(_), true) => bail!("--sqlite and --postgres are mutually exclusive"),
        (Some(path), false) => Ok(Box::new(SqliteDb::open(path)?)),
        (None, true) => {
            let user = cli
                .pg_user
                .clone()
                .ok_or_else(|| anyhow!("--pg-user is required with --postgres"))?;
            let dbname = cli
                .pg_dbname
                .clone()
                .ok_or_else(|| anyhow!("--pg-dbname is required with --postgres"))?;
            let password = match &cli.pg_password {
                Some(p) => Some(p.clone()),
                None => match std::env::var("PGPASSWORD") {
                    Ok(p) if !p.is_empty() => Some(p),
                    _ => Some(rpassword::prompt_password(format!(
                        "Password for postgres user {user}: "
                    ))?),
                },
            };
            let cfg = PgConfig {
                host: cli.pg_host.clone(),
                port: cli.pg_port,
                user,
                password,
                dbname,
            };
            Ok(Box::new(PostgresDb::connect(&cfg)?))
        }
        (None, false) => bail!("specify a database with either --sqlite <path> or --postgres (plus --pg-user/--pg-dbname/etc.)"),
    }
}

fn renderer_for(format: OutputFormat) -> Box<dyn Renderer> {
    match format {
        OutputFormat::Text => Box::new(TextRenderer),
        OutputFormat::Html => Box::new(HtmlRenderer),
    }
}

fn write_channel_log(
    path: &std::path::Path,
    network: &str,
    buffer: &str,
    rows: &[db::BacklogRow],
    renderer: &dyn Renderer,
) -> Result<()> {
    let file = fs::File::create(path)?;
    let mut writer = std::io::BufWriter::new(file);
    writer.write_all(renderer.header(network, buffer).as_bytes())?;
    for row in rows {
        let event = classify(row, buffer);
        if matches!(event, Event::Other) {
            continue;
        }
        writer.write_all(renderer.render(row.time, &event).as_bytes())?;
    }
    writer.write_all(renderer.footer().as_bytes())?;
    writer.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.channel.is_some() && cli.network.is_none() {
        bail!("--channel requires --network");
    }

    let mut db = open_db(&cli)?;

    let user_id = db
        .user_id(&cli.user)?
        .ok_or_else(|| anyhow!("user '{}' not found in the database", cli.user))?;

    let mut networks = db.networks(user_id)?;
    if let Some(wanted) = &cli.network {
        networks.retain(|n| &n.name == wanted);
        if networks.is_empty() {
            bail!("network '{wanted}' not found for user '{}'", cli.user);
        }
    }

    fs::create_dir_all(&cli.out)?;
    let renderer = renderer_for(cli.format);

    let mut total_channels = 0usize;
    let mut total_messages = 0usize;

    for network in &networks {
        let mut buffers = db.buffers(user_id, network.id)?;
        buffers.retain(|b| {
            matches!(b.buffer_type, buffer_type::CHANNEL | buffer_type::QUERY) && !b.name.trim().is_empty()
        });
        if let Some(wanted) = &cli.channel {
            buffers.retain(|b| &b.name == wanted);
        }
        if buffers.is_empty() {
            continue;
        }

        let network_dir = cli.out.join(sanitize_component(&network.name));
        fs::create_dir_all(&network_dir)?;
        let mut used_names: HashSet<String> = HashSet::new();

        for buffer in &buffers {
            let rows = db.backlog(buffer.id)?;
            let filename = unique_filename(&mut used_names, &buffer.name, renderer.extension());
            let path = network_dir.join(&filename);
            write_channel_log(&path, &network.name, &buffer.name, &rows, renderer.as_ref())?;
            println!("{}: {} messages", path.display(), rows.len());
            total_channels += 1;
            total_messages += rows.len();
        }
    }

    if let Some(channel) = &cli.channel
        && total_channels == 0
    {
        bail!(
            "channel '{}' not found in network '{}' for user '{}'",
            channel,
            cli.network.unwrap(),
            cli.user
        );
    }

    println!(
        "\nDone: {total_channels} channel(s), {total_messages} message(s) written to {}",
        cli.out.display()
    );
    Ok(())
}
