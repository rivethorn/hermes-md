import type { FileConfig, ResolvedConfig } from "./types";
import path from "path";
import fs from "fs";

export async function loadConfig(cliPath?: string): Promise<ResolvedConfig> {
  const cfg = cliPath ? await readConfig(cliPath) : {};

  const supabaseUrl = cfg.supabase_url ?? process.env.SUPABASE_URL;
  const serviceKey =
    cfg.supabase_service_key ?? process.env.SUPABASE_SERVICE_KEY;

  if (!supabaseUrl || !serviceKey) {
    throw new Error("Missing Supabase credentials");
  }

  return {
    supabaseUrl,
    serviceKey,
    bucket: cfg.bucket ?? "blog",
    table: cfg.table ?? "posts",
  };
}

async function readConfig(p: string): Promise<FileConfig> {
  const raw = await readFile(p, "utf8");
  return JSON.parse(JSON.stringify(require("toml").parse(raw)));
}

export async function genConfig(): Promise<string> {
  const p = "./config.toml";
  await mkdir(path.dirname(p), { recursive: true });

  if (fs.existsSync(p)) {
    throw new Error("Config already exists");
  }

  await writeFile(
    p,
    `supabase_url = "https://xxxxx.supabase.co"
supabase_service_key = "service_role_key"
bucket = "blog"
`,
  );

  return p;
}
function readFile(p: string, arg1: string) {
  throw new Error("Function not implemented.");
}

function mkdir(arg0: any, arg1: { recursive: boolean }) {
  throw new Error("Function not implemented.");
}

function writeFile(p: string, arg1: string) {
  throw new Error("Function not implemented.");
}
