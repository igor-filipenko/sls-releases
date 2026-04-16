CREATE TABLE IF NOT EXISTS releases (
    name TEXT NOT NULL,
    localized_name TEXT NOT NULL,
    url TEXT NOT NULL,
    date_time TEXT NOT NULL,
    version_kind TEXT NOT NULL,
    major INTEGER NOT NULL,
    minor INTEGER NOT NULL,
    patch INTEGER NOT NULL,
    rc_number INTEGER NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS releases_uniq
ON releases (name, version_kind, major, minor, patch, rc_number);

