# Hermes

> Typescript rewrite
>> Rust was NOT the best choice for this

A CLI tool for adding and removing Markdown files to and from a Supabase bucket. I used it for my blog site, you can use it for whatever you want.

## ⚠️ Warning

**Never publish your Service Role Key** — it WILL grant unlimited power over your Supabase project to anyone who has it.

## Installation

> Only Unix-based systems are supported.

You can install from NPM:

```bash
npm install -g hermes-md
# or with your package manager of choice:
bun install -g hermes-md
```

You can also download the binary from the [releases](https://github.com/rivethorn/hermes-md/releases/latest).

### Build from source

You can also compile it into a single executable:

```bash
https://github.com/rivethorn/hermes-md.git
cd hermes-md
bun install
bun run build
```

Or:

```bash
bun build src/hermes.ts --compile --outfile hermes
```

Then you can move it to somewhere in PATH environment variable, for example:

```bash
mv hermes ~/.bun/bin/hermes
```

## Usage

```bash
hermes publish <path>         # upload file + metadata
hermes list                   # show slugs and where they are (bucket/table/both)
hermes delete <slug>          # delete file + row after confirmation
hermes delete <slug> --soft   # delete only DB row (keeps bucket file)
hermes gen-config             # write sample config to where you are
```

## Configuration

### Config File (Preferred)

Place `config.toml` in the current directory (where you have your files).

Override the path with `--config /path/to/config.toml` (or `--config C:\path\to\config.toml` on Windows).

Example `config.toml`:

```toml
supabase_url = "https://xxxxx.supabase.co"
supabase_service_key = "service_role_key"
bucket = "blog"
```

### Environment Variables

Environment variables (`SUPABASE_URL`, `SUPABASE_SERVICE_KEY`, `SUPABASE_BUCKET`) are honored as a fallback if no config file is found.
