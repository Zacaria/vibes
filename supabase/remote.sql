-- Supabase database schema for cli-twitter
create extension if not exists "uuid-ossp";

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
    primary key(follower, followee)
);

create table if not exists likes (
    user_id uuid references profiles(id) on delete cascade,
    post_id uuid references posts(id) on delete cascade,
    created_at timestamptz default now(),
    primary key(user_id, post_id)
);

create table if not exists restraints (
    allowed uuid references profiles(id) on delete cascade,
    post_id uuid references posts(id) on delete cascade,
    primary key(allowed, post_id)
);

drop view if exists v_feed_public;
create view v_feed_public as
select
    p.id,
    p.author,
    p.body,
    p.audience,
    p.created_at,
    pr.handle as author_handle,
    coalesce(like_count.count, 0) as like_count
from posts p
join profiles pr on pr.id = p.author
left join lateral (
    select count(*) as count from likes where post_id = p.id
) like_count on true
where p.audience = 'public'
order by p.created_at desc;

drop view if exists v_feed_user;
create view v_feed_user as
select
    p.id,
    p.author,
    p.body,
    p.audience,
    p.created_at,
    pr.handle as author_handle,
    coalesce(like_count.count, 0) as like_count,
    f.follower as viewer
from posts p
join profiles pr on pr.id = p.author
left join follows f on f.followee = p.author
left join lateral (
    select count(*) as count from likes where post_id = p.id
) like_count on true;

create or replace function public.feed_for(uid uuid)
returns setof v_feed_user
language sql security definer as $$
    select * from v_feed_user
    where
        viewer = uid
        or author = uid
        or audience = 'public'
        or (audience = 'restrained' and exists(select 1 from restraints r where r.post_id = v_feed_user.id and r.allowed = uid));
$$;
