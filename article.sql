create table articles (
    id serial integer primary key,
    title text not null,
    content text not null,
    author text not null,
    created_at timestamp default current_timestamp
);
