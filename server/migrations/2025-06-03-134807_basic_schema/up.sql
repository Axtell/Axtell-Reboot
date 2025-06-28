-- Your SQL goes here

DROP TABLE IF EXISTS comments;
DROP TABLE IF EXISTS responses;
DROP TABLE IF EXISTS challenges;
DROP TABLE IF EXISTS challenge_types;
DROP TABLE IF EXISTS posts;
DROP TABLE IF EXISTS users;


CREATE TABLE IF NOT EXISTS posts
(
    id serial NOT NULL,
    title text NOT NULL,
    body text NOT NULL,
    user_id serial NOT NULL,
    created_at timestamp NOT NULL,
    updated_at timestamp NULL,
    deleted_at timestamp NULL,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS users
(
    id serial NOT NULL,
    name character varying NOT NULL,
    profile text NOT NULL,
    created_at timestamp NOT NULL,
    updated_at timestamp NULL,
    deleted_at timestamp NULL,
    PRIMARY KEY (id),
    CONSTRAINT users_unique_name UNIQUE (name)
);

CREATE TABLE IF NOT EXISTS challenges
(
    post_id serial NOT NULL,
    challenge_type_id smallserial NOT NULL,
    PRIMARY KEY (post_id)
);

CREATE TABLE IF NOT EXISTS responses
(
    post_id serial NOT NULL,
    challenge_id serial NOT NULL,
    code text NOT NULL,
    PRIMARY KEY (post_id),
    UNIQUE (post_id)
);

CREATE TABLE IF NOT EXISTS comments
(
    id serial NOT NULL,
    post_id serial NOT NULL,
    body character varying(256) NOT NULL,
    user_id serial NOT NULL,
    created_at timestamp NOT NULL,
    updated_at timestamp NULL,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS challenge_types
(
    id smallserial NOT NULL,
    name character varying(32) NOT NULL,
    description text NOT NULL,
    PRIMARY KEY (id),
    CONSTRAINT challenge_type_unique_name UNIQUE (name)
);

INSERT INTO challenge_types (name, description) VALUES
    ('code golf', 'shortest code wins'),
    ('other', 'specify winning criteria in challenge')
ON CONFLICT DO NOTHING;

ALTER TABLE IF EXISTS posts
    ADD FOREIGN KEY (user_id)
    REFERENCES users (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS challenges
    ADD FOREIGN KEY (post_id)
    REFERENCES posts (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS challenges
    ADD FOREIGN KEY (challenge_type_id)
    REFERENCES challenge_types (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS responses
    ADD FOREIGN KEY (post_id)
    REFERENCES posts (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS responses
    ADD FOREIGN KEY (challenge_id)
    REFERENCES challenges (post_id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS comments
    ADD FOREIGN KEY (post_id)
    REFERENCES posts (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS comments
    ADD FOREIGN KEY (user_id)
    REFERENCES users (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;