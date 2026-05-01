CREATE TABLE IF NOT EXISTS modules (
    name TEXT NOT NULL PRIMARY KEY,
    localized_name TEXT NOT NULL
);

INSERT INTO modules (name, localized_name) VALUES
    ('accumulations', 'Накопления'),
    ('bonuses', 'Бонусы'),
    ('communications', 'Коммуникации'),
    ('coupons', 'Купоны'),
    ('customers', 'Покупатели'),
    ('discounts', 'Скидки'),
    ('dwh', 'Аналитика'),
    ('favorites', 'Любимый товар'),
    ('gateway', 'Внешний API'),
    ('limits', 'Лимиты'),
    ('offers', 'Офферы'),
    ('purchases', 'Чеки'),
    ('registrations', 'Регистрации'),
    ('segments', 'Сегменты'),
    ('triggers', 'Триггеры'),
    ('scheduler', 'Планировщик'),
    ('superset', 'Superset'),
    ('superset-integration', 'Superset Интеграция'),
    ('frontend-loyalty', 'Фронтенд Лояльности'),
    ('frontend-registrations', 'Фронтенд Регистрации');

CREATE TABLE IF NOT EXISTS releases (
    name TEXT NOT NULL REFERENCES modules (name),
    url TEXT NOT NULL,
    date_time TEXT NOT NULL,
    version_kind TEXT NOT NULL,
    major INTEGER NOT NULL,
    minor INTEGER NOT NULL,
    patch INTEGER NOT NULL,
    rc_number INTEGER NOT NULL DEFAULT 0,
    closed BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE UNIQUE INDEX IF NOT EXISTS releases_uniq
ON releases (name, version_kind, major, minor, patch, rc_number);
