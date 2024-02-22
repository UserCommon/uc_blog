create table articles (
    id integer primary key autoincrement,
    title text not null,
    content text not null,
    author text not null,
    created_at timestamp default current_timestamp not null
);
