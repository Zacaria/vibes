-- Enable row level security
alter table profiles enable row level security;
alter table posts enable row level security;
alter table follows enable row level security;
alter table likes enable row level security;
alter table restraints enable row level security;

-- Profiles policies
create policy "profiles_select" on profiles
    for select using (auth.role() = 'authenticated');

create policy "profiles_insert" on profiles
    for insert with check (auth.uid() = id);

create policy "profiles_update" on profiles
    for update using (auth.uid() = id)
    with check (auth.uid() = id);

-- Posts policies
create policy "posts_select" on posts
    for select using (
        audience = 'public'
        or author = auth.uid()
        or (
            audience = 'restrained'
            and exists(select 1 from restraints r where r.post_id = posts.id and r.allowed = auth.uid())
        )
        or (audience = 'private' and author = auth.uid())
    );

create policy "posts_insert" on posts
    for insert with check (author = auth.uid());

create policy "posts_modify" on posts
    for update using (author = auth.uid())
    with check (author = auth.uid());

create policy "posts_delete" on posts
    for delete using (author = auth.uid());

-- Follows policies
create policy "follows_rw" on follows
    for all using (follower = auth.uid())
    with check (follower = auth.uid());

-- Likes policies
create policy "likes_rw" on likes
    for all using (user_id = auth.uid())
    with check (user_id = auth.uid());

-- Restraints policies
create policy "restraints_rw" on restraints
    for all using (allowed = auth.uid() or exists(select 1 from posts p where p.id = restraints.post_id and p.author = auth.uid()))
    with check (allowed = auth.uid() or exists(select 1 from posts p where p.id = restraints.post_id and p.author = auth.uid()));
