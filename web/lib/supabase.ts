import { createClient } from "@supabase/supabase-js";

const supabaseUrl = process.env.NEXT_PUBLIC_SUPABASE_URL!;
const supabaseAnonKey = process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY!;

// Browser-safe client (uses anon key, subject to RLS)
export const supabase = createClient(supabaseUrl, supabaseAnonKey);

// Server-side client for API routes — uses service role key if available,
// falls back to anon key (RLS still applies for anon key).
export function createServerClient() {
  const key =
    process.env.SUPABASE_SERVICE_ROLE_KEY ?? process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY!;
  return createClient(supabaseUrl, key, {
    auth: { persistSession: false },
  });
}
