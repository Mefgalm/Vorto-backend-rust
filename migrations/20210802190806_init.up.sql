-- Add up migration script here
-- teams
CREATE TABLE teams (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL
);

CREATE UNIQUE INDEX teams_name_index ON teams (name);

-- users
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP(0) NOT NULL
);
CREATE UNIQUE INDEX users_login_index ON users (email);

-- vocs
CREATE TABLE vocs (
    id SERIAL PRIMARY KEY,
    short VARCHAR(255) NOT NULL,
    "full" VARCHAR(255) NOT NULL
);
CREATE UNIQUE INDEX vocs_full_index ON vocs ("full");
CREATE UNIQUE INDEX vocs_short_index ON vocs (short);

-- words
CREATE TABLE words (
    id SERIAL PRIMARY KEY,
    body VARCHAR(255) NOT NULL,
    status VARCHAR(255) NOT NULL,
    is_edited_after_load BOOLEAN NOT NULL,
    load_status VARCHAR(255) NOT NULL,
    difficulty INT NOT NULL,
    "timestamp" BIGINT NOT NULL
);
CREATE UNIQUE INDEX words_body_index ON words (body);

-- word definitions
CREATE TABLE word_definitions (
    id SERIAL PRIMARY KEY,
    definition VARCHAR(1000) NOT NULL,
    status VARCHAR(255) NOT NULL,
    "order" INT NOT NULL,
    word_id INT NOT NULL,
    voc_id INT,
    CONSTRAINT word_definitions_voc_id_fkey 
        FOREIGN KEY (voc_id)
        REFERENCES vocs (id),
    CONSTRAINT word_definitions_word_id_fkey
        FOREIGN KEY (word_id)
        REFERENCES words (id)
);
CREATE INDEX word_definitions_word_id_index ON word_definitions (word_id);
CREATE INDEX word_definitions_voc_id_index ON word_definitions (voc_id);
CREATE UNIQUE INDEX word_id_definition_unique ON word_definitions (word_id,definition);

-- games
CREATE TABLE games (
	id SERIAL PRIMARY KEY,
	state VARCHAR(255) NOT NULL,
	created_at TIMESTAMP(0) NOT NULL,
	expired_at TIMESTAMP(0) NOT NULL,
	word_count INT NOT NULL,
	penalty BOOLEAN NOT NULL,
	round_time INT NOT NULL,
	winner_id INT NULL,
	turn INT NOT NULL,
	"token" VARCHAR(255) NOT NULL
);

-- team results
CREATE TABLE team_results (
	id SERIAL PRIMARY KEY,
	team_id INT NOT NULL,
	game_id INT NOT NULL,
	"order" INT NOT NULL, 
	CONSTRAINT team_results_team_id_fkey 
		FOREIGN KEY (team_id)
		REFERENCES teams (id),
 	CONSTRAINT team_results_game_id_fkey
 		FOREIGN KEY (game_id)
 		REFERENCES games (id)
);
CREATE INDEX team_results_game_id_index ON team_results (game_id);
CREATE INDEX team_results_team_id_index ON team_results (team_id);

ALTER TABLE games ADD CONSTRAINT games_winner_id_fkey FOREIGN KEY (winner_id) REFERENCES team_results(id);

-- word results
CREATE TABLE word_results (
	id serial PRIMARY KEY,
	"result" BOOLEAN NOT NULL,
	word_id INT NOT NULL,
	"order" INT NOT NULL,
	team_result_id INT NOT NULL,
	CONSTRAINT word_results_word_id_fkey
		FOREIGN KEY (word_id)
		REFERENCES words(id),
	CONSTRAINT word_results_team_result_id_fkey
		FOREIGN KEY (team_result_id)
		REFERENCES team_results (id)
);
CREATE INDEX word_results_team_result_id_index ON word_results (team_result_id);
CREATE INDEX word_results_word_id_index ON word_results (word_id);