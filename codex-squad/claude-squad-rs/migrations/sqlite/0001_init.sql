create table if not exists conversations (
    id text primary key,
    title text not null,
    created_at text not null,
    updated_at text not null
);

create table if not exists messages (
    id text primary key,
    conversation_id text not null references conversations(id) on delete cascade,
    role text not null,
    sender text not null,
    body text not null,
    created_at text not null
);

create index if not exists idx_messages_conversation on messages(conversation_id);
