# quassel-dump-all

Dumps a Quassel IRC user's logs out of a Quassel core database and into a
plain directory tree, one file per channel/query buffer, grouped into a
directory per network.

Reads either a Quassel sqlite3 database file or connects directly to a
Quassel PostgreSQL database.

## Output layout

```
quassel-dump-all-out/
├── libera.chat/
│   ├── #perl.log
│   └── #moose.log
└── oftcnet/
    └── #debian.log
```

Only channel buffers and query (private message) buffers belonging to the
given user are dumped; status buffers are skipped. Network and channel names
are sanitized into safe filenames, and colliding names are disambiguated
with a numeric suffix.

Each line is timestamped in the local timezone and formatted mIRC-log style,
e.g.:

```
[2017-06-26 15:09:25] <perigrin> would that be like Acme::LookOfDisapproval?
[2017-06-26 15:09:43] *** tchaves (~chaves@200.202.254.34) has quit (Quit: Leaving.)
```

Joins, parts, quits, kicks, kills, nick changes, mode changes, topic
changes, and server/error/info/day-change lines are all recognized and
formatted; anything else is skipped.

Pass `-f html` to get one styled, self-contained HTML file per channel
instead (same content, colored by event type, with light/dark support)
rather than a plain `.log` file.

## Building

```
cargo build --release
```

The resulting binary is `target/release/quassel-dump-all`.

## Usage

```
quassel-dump-all -u <username> [options] (--sqlite <path> | --postgres ...)
```

| Flag | Description |
| --- | --- |
| `-u`, `--user <USER>` | Quassel username whose logs will be dumped (required) |
| `-o`, `--out <DIR>` | Output directory (default: `quassel-dump-all-out`) |
| `-f`, `--format <text\|html>` | Output format (default: `text`) |
| `-n`, `--network <NETWORK>` | Only dump this network |
| `-c`, `--channel <CHANNEL>` | Only dump this channel/buffer (requires `-n`) |

### Database source

Exactly one of the following must be given:

**SQLite**

| Flag | Description |
| --- | --- |
| `--sqlite <PATH>` | Path to a Quassel `quassel-storage.sqlite` file |

**PostgreSQL**

| Flag | Description |
| --- | --- |
| `--postgres` | Connect to PostgreSQL instead of sqlite |
| `--pg-host <HOST>` | Host (default: `localhost`) |
| `--pg-port <PORT>` | Port (default: `5432`) |
| `--pg-user <USER>` | Username (required) |
| `--pg-dbname <NAME>` | Database name (required) |
| `--pg-password <PASSWORD>` | Password (optional; see below) |

If `--pg-password` is omitted, the `PGPASSWORD` environment variable is used
if set, otherwise you're prompted for it interactively. The connection is
made without TLS, so this is intended for local or otherwise trusted
connections.

## Examples

Dump everything for a user out of a local sqlite database, as text:

```
quassel-dump-all -u alice --sqlite ~/.local/share/quassel/quassel-storage.sqlite
```

Same, but as HTML, into a custom directory:

```
quassel-dump-all -u alice -f html -o ~/irc-logs \
  --sqlite ~/.local/share/quassel/quassel-storage.sqlite
```

Dump only one channel on one network from a PostgreSQL-backed core:

```
quassel-dump-all -u alice -n libera.chat -c '#perl' \
  --postgres --pg-host db.example.com --pg-user quassel --pg-dbname quassel
```

## License

MIT. See [LICENSE](LICENSE).
