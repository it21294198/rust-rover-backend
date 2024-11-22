-- 1. Create the todo table with id as TEXT
CREATE TABLE IF NOT EXISTS todo (
    id TEXT PRIMARY KEY,
    todo TEXT NOT NULL,
    status INT NOT NULL
);

-- 2. Create the insert_one_todo stored procedure
CREATE OR REPLACE PROCEDURE insert_one_todo(
    p_id TEXT,
    p_todo TEXT,
    p_status INT,
    OUT o_id TEXT,
    OUT o_todo TEXT,
    OUT o_status INT
)
LANGUAGE plpgsql
AS $$
BEGIN
    INSERT INTO todo (id, todo, status)
    VALUES (p_id, p_todo, p_status)
    RETURNING id, todo, status INTO o_id, o_todo, o_status;
END;
$$;

-- 3. Create the update_one_todo stored procedure
CREATE OR REPLACE PROCEDURE update_one_todo(
    p_id TEXT,
    p_todo TEXT,
    p_status INT,
    OUT o_id TEXT,
    OUT o_todo TEXT,
    OUT o_status INT
)
LANGUAGE plpgsql
AS $$
BEGIN
    UPDATE todo
    SET todo = p_todo, status = p_status
    WHERE id = p_id
    RETURNING id, todo, status INTO o_id, o_todo, o_status;
END;
$$;

-- 4. Create the delete_todo stored procedure
CREATE OR REPLACE PROCEDURE delete_todo(
    p_id TEXT,
    OUT o_deleted BOOLEAN
)
LANGUAGE plpgsql
AS $$
BEGIN
    DELETE FROM todo WHERE id = p_id;
    
    IF FOUND THEN
        o_deleted := TRUE;
    ELSE
        o_deleted := FALSE;
    END IF;
END;
$$;

-- 5. Insert fake data into the todo table
INSERT INTO todo (id, todo, status)
VALUES 
    (md5(random()::text), 'Buy groceries', 1),
    (md5(random()::text), 'Finish project report', 0),
    (md5(random()::text), 'Clean the house', 1),
    (md5(random()::text), 'Read a book', 0),
    (md5(random()::text), 'Exercise for 30 minutes', 1);

-- *for rover --------------------------------------------------

drop table test;
DROP PROCEDURE insert_one_test(text,text);

-- Create table for operations
CREATE TABLE test (
    id TEXT PRIMARY KEY,
    metadata JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE OR REPLACE PROCEDURE insert_one_test(
    p_id TEXT,
    p_metadata TEXT,
    OUT p_result TEXT
)
LANGUAGE plpgsql
AS $$
DECLARE
    validated_json JSONB;
BEGIN
    -- Validate and parse the input as JSON
    BEGIN
        validated_json := p_metadata::JSONB;
    EXCEPTION 
        WHEN OTHERS THEN
            RAISE EXCEPTION 'Invalid JSON input: %', p_metadata;
    END;

    -- Insert the operation with validated JSON
    INSERT INTO test (id, metadata)
    VALUES (p_id, validated_json);

    p_result := '1';
END;
$$;

CALL insert_one_test(
    'unique_id_1', 
    '{"key": "value", "another_key": "another_value"}'
);
