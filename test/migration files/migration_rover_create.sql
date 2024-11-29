drop TABLE rovers;

CREATE TABLE rovers (
    rover_id SERIAL PRIMARY KEY,
    initial_id INTEGER NOT NULL,
    rover_status INTEGER NOT NULL,
	user_id INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

DROP PROCEDURE public.create_new_rover(in int4, in int4, in int4, out text);

CREATE OR REPLACE PROCEDURE create_new_rover(
    initial_id INTEGER,
    rover_status INTEGER,
    user_id INTEGER,
    OUT result_status BOOLEAN
)
LANGUAGE plpgsql
AS $$
BEGIN
    -- Insert the values into the rovers table
    INSERT INTO rovers (initial_id, rover_status, user_id)
    VALUES (initial_id, rover_status, user_id);
    
    -- If the insert succeeds, set the result status to true
    result_status := TRUE;

EXCEPTION
    WHEN OTHERS THEN
        -- Log the error and set the result status to false
        RAISE NOTICE 'Insertion failed: %', SQLERRM;
        result_status := FALSE;
END;
$$;


DO $$
DECLARE
    result_id BOOLEAN; -- Declare a variable to capture the OUT parameter
BEGIN
    -- Call the procedure
    CALL create_new_rover(1, 2, 3, result_id);

    -- Display the result
    RAISE NOTICE 'Generated Result ID: %', result_id;
    -- output
    -- Generated Result ID: t
END;
$$;

-- insert into rovers values (1234,1,4321,1234);
-- insert into rovers (initial_id,rover_status,user_id) values (1234,1,4321);
