import path from "path";

export function normalizeSlug(input: string): string {
  return path.basename(input, path.extname(input));
}
