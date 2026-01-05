import slugify from "slugify";
import path from "path";
import { readFile } from "fs/promises";

import { supabaseClient } from "./supabase";
import type { ResolvedConfig } from "./types";
import { parseFrontmatter } from "./frontmatter";

export async function publish(cfg: ResolvedConfig, filePath: string) {
  const md = await readFile(filePath, "utf8");

  const { frontmatter } = parseFrontmatter(md);
  if (!frontmatter) {
    throw new Error("Missing or invalid frontmatter");
  }

  const slug =
    frontmatter.slug ??
    slugify(path.basename(filePath, path.extname(filePath)), { lower: true });

  const supabase = supabaseClient(cfg);

  /* Upload markdown */
  const { error: uploadErr } = await supabase.storage
    .from(cfg.bucket)
    .upload(`${slug}.md`, md, {
      upsert: true,
      contentType: "text/markdown",
    });

  if (uploadErr) throw uploadErr;

  /* Upsert metadata */
  const { error: dbErr } = await supabase.from(cfg.table).upsert(
    {
      slug,
      title: frontmatter.title,
      tag: frontmatter.tag,
      time_to_read: frontmatter.ttr,
      summary: frontmatter.summary,
    },
    { onConflict: "slug" },
  );

  if (dbErr) throw dbErr;

  console.log(`✓ Published: ${frontmatter.title}`);
}
