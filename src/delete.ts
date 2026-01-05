import { supabaseClient } from "./supabase";
import type { ResolvedConfig } from "./types";
import { normalizeSlug } from "./utils";

export async function deletePost(
  cfg: ResolvedConfig,
  inputSlug: string,
  soft = false,
) {
  const slug = normalizeSlug(inputSlug);
  const supabase = supabaseClient(cfg);

  const { data: file } = await supabase.storage
    .from(cfg.bucket)
    .list("", { search: `${slug}.md` });

  const inStorage = !!file?.length;

  const { data: rows } = await supabase
    .from(cfg.table)
    .select("slug")
    .eq("slug", slug);

  const inTable = !!rows?.length;

  if (!inStorage && !inTable) {
    throw new Error(`Slug '${slug}' not found`);
  }

  if (!soft && inStorage) {
    const { error } = await supabase.storage
      .from(cfg.bucket)
      .remove([`${slug}.md`]);
    if (error) throw error;
  }

  if (inTable) {
    const { error } = await supabase.from(cfg.table).delete().eq("slug", slug);
    if (error) throw error;
  }

  console.log(`✓ Deleted ${slug}`);
}
