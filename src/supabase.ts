import { SupabaseClient, createClient } from "@supabase/supabase-js";
import type { ResolvedConfig } from "./types";

export function supabaseClient(cfg: ResolvedConfig): SupabaseClient {
  return createClient(cfg.supabaseUrl, cfg.serviceKey, {
    auth: { persistSession: false },
  });
}
