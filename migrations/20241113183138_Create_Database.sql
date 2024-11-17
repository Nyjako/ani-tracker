
CREATE TABLE Anime
(
  id                INTEGER      NOT NULL UNIQUE,
  name              VARCHAR(255) NOT NULL,
  path              VARCHAR(255) NOT NULL,
  root_directory_id INTEGER      NOT NULL,
  PRIMARY KEY (id AUTOINCREMENT),
  FOREIGN KEY (root_directory_id) REFERENCES Directory (id)
);

CREATE TABLE Directory
(
  id       INTEGER      NOT NULL UNIQUE,
  path     VARCHAR(255) NOT NULL,
  -- 0 - To watch, 1 - Watched | Directories would be automatically moved to watched when you finish anime
  dir_type INTEGER      NOT NULL DEFAULT 0,
  PRIMARY KEY (id AUTOINCREMENT)
);

CREATE TABLE Episode
(
  id       INTEGER      NOT NULL UNIQUE,
  name     VARCHAR(255) NULL    ,
  anime_id INTEGER      NOT NULL,
  path     VARCHAR(255) NOT NULL,
  PRIMARY KEY (id AUTOINCREMENT),
  FOREIGN KEY (anime_id) REFERENCES Anime (id)
);
