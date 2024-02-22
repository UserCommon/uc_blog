create table articles (
    id serial integer,
    title text not null,
    content text not null,
    author text not null,
    created_at timestamp default current_timestamp
);
