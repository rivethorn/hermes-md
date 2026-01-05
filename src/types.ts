export interface FrontMatter {
  title: string;
  tag: string;
  ttr: string;
  slug: string;
  summary: string;
}

export interface FileConfig {
  supabase_url?: string;
  supabase_service_key?: string;
  bucket?: string;
  table?: string;
}

export interface ResolvedConfig {
  supabaseUrl: string;
  serviceKey: string;
  bucket: string;
  table: string;
}
