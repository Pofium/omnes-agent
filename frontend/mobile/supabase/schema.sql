-- OmnesAgent Supabase Schema Migration (§2.1 PLAN_adbot-to-omnes-agent.md)
-- Run this script in the Supabase SQL Editor

-- 1. Profiles table
create table if not exists public.profiles (
  id uuid primary key references auth.users(id) on delete cascade,
  display_name text,
  avatar_url text,
  email text,
  created_at timestamp with time zone default timezone('utc'::text, now()) not null,
  updated_at timestamp with time zone default timezone('utc'::text, now()) not null
);

alter table public.profiles enable row level security;

create policy "Public profiles are viewable by authenticated users"
  on public.profiles for select
  to authenticated
  using (true);

create policy "Users can insert their own profile"
  on public.profiles for insert
  to authenticated
  with check (auth.uid() = id);

create policy "Users can update their own profile"
  on public.profiles for update
  to authenticated
  using (auth.uid() = id);

-- 2. Suggested Prompts table (replacing suggested_category in Firestore)
create table if not exists public.suggested_prompts (
  id uuid primary key default gen_random_uuid(),
  category text not null,
  title text not null,
  prompt text not null,
  sort integer default 0,
  created_at timestamp with time zone default timezone('utc'::text, now()) not null
);

alter table public.suggested_prompts enable row level security;

create policy "Authenticated users can read suggested prompts"
  on public.suggested_prompts for select
  to authenticated
  using (true);

-- Insert default starter prompts
insert into public.suggested_prompts (category, title, prompt, sort)
values
  ('Общие', 'Создай план проекта', 'Помоги создать подробный пошаговый план нового проекта', 1),
  ('Общие', 'Сводка новостей', 'Подготовь краткую сводку последних обновлений', 2),
  ('Код', 'Напиши скрипт автоматизации', 'Напиши Bash/Python скрипт для регулярного бэкапа рабочей директории', 3),
  ('Файлы', 'Проанализируй воркспейс', 'Посмотри файлы в текущей директории и расскажи об их структуре', 4)
on conflict do nothing;

-- 3. User Settings table (per-user sync across devices)
create table if not exists public.user_settings (
  user_id uuid primary key references auth.users(id) on delete cascade,
  settings jsonb not null default '{}'::jsonb,
  updated_at timestamp with time zone default timezone('utc'::text, now()) not null
);

alter table public.user_settings enable row level security;

create policy "Users can manage their own settings"
  on public.user_settings for all
  to authenticated
  using (auth.uid() = user_id)
  with check (auth.uid() = user_id);

-- 4. Avatars storage bucket setup
insert into storage.buckets (id, name, public)
values ('avatars', 'avatars', true)
on conflict (id) do nothing;

create policy "Avatar images are publicly accessible"
  on storage.objects for select
  using (bucket_id = 'avatars');

create policy "Users can upload their own avatar"
  on storage.objects for insert
  to authenticated
  with check (bucket_id = 'avatars' and (storage.foldername(name))[1] = auth.uid()::text);

create policy "Users can update their own avatar"
  on storage.objects for update
  to authenticated
  using (bucket_id = 'avatars' and (storage.foldername(name))[1] = auth.uid()::text);
