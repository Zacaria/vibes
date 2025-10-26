-- Schema for cli-twitter Supabase project
create extension if not exists "uuid-ossp";
create extension if not exists pgcrypto;

create table if not exists profiles (
    id uuid primary key default uuid_generate_v4(),
    handle text unique not null,
    display_name text,
    created_at timestamptz default now()
);

create table if not exists posts (
    id uuid primary key default gen_random_uuid(),
    author uuid references profiles(id) on delete cascade,
    body text not null check (char_length(body) <= 280),
    audience text not null check (audience in ('public','restrained','private')),
    created_at timestamptz default now()
);

create table if not exists follows (
    follower uuid references profiles(id) on delete cascade,
    followee uuid references profiles(id) on delete cascade,
    created_at timestamptz default now(),
    primary key (follower, followee)
);

create table if not exists likes (
    user_id uuid references profiles(id) on delete cascade,
    post_id uuid references posts(id) on delete cascade,
    created_at timestamptz default now(),
    primary key (user_id, post_id)
);

create table if not exists restraints (
    allowed uuid references profiles(id) on delete cascade,
    post_id uuid references posts(id) on delete cascade,
    primary key (allowed, post_id)
);

create or replace view public.v_feed_public as
select p.id,
       p.author,
       p.body,
       p.audience,
       p.created_at,
       prof.handle as author_handle,
       coalesce(lc.count, 0) as like_count
from posts p
left join profiles prof on prof.id = p.author
left join (
    select post_id, count(*) as count from likes group by post_id
) lc on lc.post_id = p.id
where p.audience = 'public'
order by p.created_at desc;

create or replace function public.feed_global(uid uuid)
returns setof v_feed_public
language sql stable
as $$
    select * from v_feed_public limit 200;
$$;

create or replace function public.feed_me(uid uuid)
returns table (
    id uuid,
    author uuid,
    body text,
    audience text,
    created_at timestamptz,
    author_handle text,
    like_count bigint,
    liked boolean
)
language sql stable
as $$
    select p.id,
           p.author,
           p.body,
           p.audience,
           p.created_at,
           prof.handle,
           coalesce(lc.count,0) as like_count,
           exists(select 1 from likes l where l.post_id = p.id and l.user_id = uid) as liked
    from posts p
    left join profiles prof on prof.id = p.author
    left join (
        select post_id, count(*) as count from likes group by post_id
    ) lc on lc.post_id = p.id
    where p.author = uid
    order by p.created_at desc
    limit 200;
$$;

create or replace function public.feed_following(uid uuid)
returns table (
    id uuid,
    author uuid,
    body text,
    audience text,
    created_at timestamptz,
    author_handle text,
    like_count bigint,
    liked boolean
)
language sql stable
as $$
    select p.id,
           p.author,
           p.body,
           p.audience,
           p.created_at,
           prof.handle,
           coalesce(lc.count,0) as like_count,
           exists(select 1 from likes l where l.post_id = p.id and l.user_id = uid) as liked
    from posts p
    join follows f on f.followee = p.author
    left join profiles prof on prof.id = p.author
    left join (
        select post_id, count(*) as count from likes group by post_id
    ) lc on lc.post_id = p.id
    where f.follower = uid
       or p.author = uid
       or p.audience = 'public'
       or (p.audience = 'restrained' and exists(select 1 from restraints r where r.post_id = p.id and r.allowed = uid))
       or (p.audience = 'private' and p.author = uid)
    order by p.created_at desc
    limit 200;
$$;
