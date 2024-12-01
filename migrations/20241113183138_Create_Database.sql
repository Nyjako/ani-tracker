
-- Stores information about anime titles.
CREATE TABLE anime
(
  -- Primary key.
  id             INTEGER NOT NULL UNIQUE,
  -- MyAnimeList ID (might be null if name was not found)
  mal_id         INTEGER NOT NULL UNIQUE,
  -- Name of the anime. (Local name would be used if not found on mal)
  name           TEXT    NOT NULL,
  -- Description of the anime.
  description    TEXT    NULL    ,
  -- Total number of episodes.
  total_episodes INTEGER NOT NULL,
  -- Anime cover art
  cover_art      BLOB    NULL    ,
  -- Lower resolution of cover art
  thumbnail      BLOB    NULL    ,
  -- Watched status (e.g., watching, completed).
  status         TEXT    NOT NULL,
  -- Foreign key referencing directories(id).
  directory_id   INTEGER NOT NULL,
  PRIMARY KEY (id, mal_id),
  FOREIGN KEY (directory_id) REFERENCES directories (id)
);

-- Tracks directories containing anime files.
CREATE TABLE directories
(
  -- Primary key.
  id   INTEGER NOT NULL UNIQUE,
  -- Path to the directory.
  path TEXT    NOT NULL,
  -- Type of directory (e.g., watching, watched).
  dir_type TEXT    NOT NULL,
  PRIMARY KEY (id AUTOINCREMENT)
);

-- Stores information about individual episodes of an anime.
CREATE TABLE episodes
(
  -- Primary key.
  id             INTEGER NOT NULL UNIQUE,
  -- Foreign key referencing anime(id).
  anime_id       INTEGER NOT NULL,
  -- The episodes number.
  episode_number INTEGER NOT NULL,
  -- Path to the episode file on disk.
  file_path      TEXT    NOT NULL,
  -- Length of the episode in seconds.
  length         INTEGER NOT NULL,
  -- Total watched time in seconds.
  watched_time   INTEGER NOT NULL DEFAULT 0,
  -- Whether the episode has been fully watched.
  is_watched     INTEGER NOT NULL DEFAULT FALSE,
  -- Will store episode thumbnail after it is generated
  thumbnail      BLOB    NULL    ,
  PRIMARY KEY (id AUTOINCREMENT),
  FOREIGN KEY (anime_id) REFERENCES anime (id)
);

-- Stores MAL tokens
CREATE TABLE mal_settings
(
  mal_access_token     TEXT    NOT NULL UNIQUE,
  mal_refresh_token    TEXT    NOT NULL UNIQUE,
  -- Unix time
  mal_token_expires_at INTEGER NOT NULL,
  PRIMARY KEY (mal_access_token, mal_refresh_token)
);

-- Stores application settings or metadata.
CREATE TABLE settings
(
  -- Setting name.
  key   TEXT NOT NULL UNIQUE,
  -- 	Setting value.
  value TEXT NOT NULL,
  PRIMARY KEY (key)
);
