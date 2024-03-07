create table articles (
    id integer primary key autoincrement not null,
    title text not null unique,
    content text not null,
    created_at timestamp default current_timestamp not null
);
