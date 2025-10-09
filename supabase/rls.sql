-- Enable row level security
alter table profiles enable row level security;
alter table posts enable row level security;
alter table follows enable row level security;
alter table likes enable row level security;
alter table restraints enable row level security;

-- Profiles policies
create policy "Profiles are readable" on profiles
    for select using (auth.role() = 'authenticated');
create policy "Users manage own profile" on profiles
    for insert with check (id = auth.uid());
create policy "Update own profile" on profiles
    for update using (id = auth.uid()) with check (id = auth.uid());

-- Posts policies
create policy "Read public posts" on posts
    for select using (
        audience = 'public'
        or author = auth.uid()
        or (
            audience = 'restrained'
            and exists(select 1 from restraints r where r.post_id = posts.id and r.allowed = auth.uid())
        )
        or (
            audience = 'private' and author = auth.uid()
        )
    );
create policy "Insert own posts" on posts
    for insert with check (author = auth.uid());
create policy "Update own posts" on posts
    for update using (author = auth.uid()) with check (author = auth.uid());
create policy "Delete own posts" on posts
    for delete using (author = auth.uid());

-- Follows
create policy "Manage follows" on follows
    for all using (follower = auth.uid()) with check (follower = auth.uid());

-- Likes
create policy "Manage likes" on likes
    for all using (user_id = auth.uid()) with check (user_id = auth.uid());

-- Restraints
create policy "Author manages restraints" on restraints
    for all using (
        exists(select 1 from posts p where p.id = restraints.post_id and p.author = auth.uid())
    ) with check (
        exists(select 1 from posts p where p.id = restraints.post_id and p.author = auth.uid())
    );
