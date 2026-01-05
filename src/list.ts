import { supabaseClient } from "./supabase";
import type { ResolvedConfig } from "./types";
import { normalizeSlug } from "./utils";

export async function listItems(cfg: ResolvedConfig) {
  const supabase = supabaseClient(cfg);

  const { data: files } = await supabase.storage.from(cfg.bucket).list();

  const storageSlugs = new Set(files?.map((f) => normalizeSlug(f.name)) ?? []);

  const { data: rows } = await supabase.from(cfg.table).select("slug");

  const tableSlugs = new Set(rows?.map((r) => normalizeSlug(r.slug)) ?? []);

  const all = new Set([...storageSlugs, ...tableSlugs]);

  if (all.size === 0) {
    console.log("No slugs found.");
    return;
  }

  console.log("slug".padEnd(32), "location");
  [...all].sort().forEach((slug) => {
    const loc =
      storageSlugs.has(slug) && tableSlugs.has(slug)
        ? "both"
        : storageSlugs.has(slug)
          ? "bucket"
          : "table";
    console.log(slug.padEnd(32), loc);
  });
}
