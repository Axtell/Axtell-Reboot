-- This file should undo anything in `up.sql`

DROP TABLE IF EXISTS comments;
DROP TABLE IF EXISTS responses;
DROP TABLE IF EXISTS challenges;
DROP TABLE IF EXISTS challenge_types;
DROP TABLE IF EXISTS posts;
DROP TABLE IF EXISTS users;

DROP TYPE IF EXISTS post_type;