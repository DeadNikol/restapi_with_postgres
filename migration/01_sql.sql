-- Создаём таблицу B (справочник)
CREATE TABLE IF NOT EXISTS table_b (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Создаём таблицу A (основная)
CREATE TABLE IF NOT EXISTS table_a (
    id SERIAL PRIMARY KEY,
    value TEXT NOT NULL,
    b_id INTEGER REFERENCES table_b(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Добавляем начальные данные в B
INSERT INTO table_b (name) VALUES 
    ('Первая категория'),
    ('Вторая категория')
ON CONFLICT (id) DO NOTHING;

-- Добавляем начальные данные в A
INSERT INTO table_a (value, b_id) VALUES 
    ('Значение 1', 1),
    ('Значение 2', 1),
    ('Значение 3', 2)
ON CONFLICT (id) DO NOTHING;