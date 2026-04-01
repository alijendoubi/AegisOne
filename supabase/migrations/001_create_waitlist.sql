-- Waitlist table
create table if not exists public.waitlist (
  id          uuid primary key default gen_random_uuid(),
  email       text not null,
  platform    text not null default 'Windows',
  source      text,
  referrer    text,
  created_at  timestamptz not null default now()
);

-- Unique constraint to prevent duplicate sign-ups
alter table public.waitlist
  add constraint waitlist_email_unique unique (email);

-- Index for email lookups
create index if not exists waitlist_email_idx on public.waitlist (email);

-- Row Level Security
alter table public.waitlist enable row level security;

-- Anon users can insert only (no reads, no updates, no deletes)
create policy "allow_anon_insert" on public.waitlist
  for insert
  to anon
  with check (true);

-- Service role bypasses RLS so admin queries work as expected
-- (Supabase service role already bypasses RLS by default)
