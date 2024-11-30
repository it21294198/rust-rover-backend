drop table operations;

CREATE TABLE operations (
    id SERIAL PRIMARY KEY,
    rover_id INTEGER NOT NULL,
    random_id INTEGER NOT NULL,
    battery_status FLOAT NOT NULL,
    temp FLOAT NOT NULL,
    humidity FLOAT NOT NULL,
    result_image TEXT,
    image_data TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

DROP PROCEDURE public.insert_one_operation(in int4, in int4, in float8, in float8, in float8, in text , in text , out text);

CREATE OR REPLACE PROCEDURE insert_one_operation(
    rover_id INTEGER,
    random_id INTEGER,
    battery_status FLOAT,
    temp FLOAT,
    humidity FLOAT,
    result_image TEXT,
    image_data TEXT,
    OUT result TEXT
)
LANGUAGE plpgsql
AS $$
BEGIN
    INSERT INTO operations (
        rover_id, random_id, battery_status, temp, humidity, result_image,image_data
    )
    VALUES (
        rover_id, random_id, battery_status, temp, humidity, result_image,image_data
    );

    -- Set the OUT parameter to '1' on success
    result := '1';
EXCEPTION
    WHEN OTHERS THEN
        -- Handle exceptions by returning an appropriate message or error code
        result := 'Insertion failed: ' || SQLERRM;
END;
$$;

DO $$
DECLARE
    result TEXT; -- Declare a variable to capture the OUT parameter
BEGIN
    -- Call the procedure
    CALL insert_one_operation(1, 102, 85.5, 36.7, 60.2, '#qwerty','{"image": "data"}',result);

    -- Display the result
    RAISE NOTICE 'Generated Result ID: %', result;
    -- output
    -- Generated Result ID: t
END;
$$;

DO $$
DECLARE
    result TEXT; -- Declare a variable to capture the OUT parameter
BEGIN
    -- Call the procedure
    CALL insert_one_operation(2, 102, 85.5, 36.7, 60.2, '#qwerty','{"image": "data","test":1}',result);

    -- Display the result
    RAISE NOTICE 'Generated Result ID: %', result;
    -- output
    -- Generated Result ID: t
END;
$$;

SELECT image_data FROM operations;

SELECT image_data::json->>'image' AS image_content
FROM operations
WHERE rover_id = 1;

SELECT image_data FROM operations WHERE image_data::json->>'test' = '1';

-- sample updates
UPDATE operations
SET image_data = 
    LEFT(image_data, LENGTH(image_data) - 1) || ',"cost":100}'
WHERE image_data::json->>'test' = '1';

SELECT image_data FROM operations WHERE image_data::json->>'test' = '1';

UPDATE operations
SET image_data = REGEXP_REPLACE(
    image_data,
    '("cost":)[^,}]*',
    '\1 200',
    'g'
)
WHERE image_data::json->>'test' = '1';

SELECT image_data FROM operations WHERE image_data::json->>'test' = '1';