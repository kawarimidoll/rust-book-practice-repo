INSERT INTO
    roles (name)
VALUES
    ('Admin'),
    ('User') ON CONFLICT DO NOTHING;

INSERT INTO
    users (name, email, password_hash, role_id)
SELECT
    'Eleazar Fig',
    'eleazar.fig@example.com',
    '$2b$12$P3w8D7yJLurb/jCjR5c0de8KNkNmG2BcCbVTUVvUOrmQBzNlNKcJC',
    -- password: password
    role_id
FROM
    roles
WHERE
    name LIKE 'Admin';
