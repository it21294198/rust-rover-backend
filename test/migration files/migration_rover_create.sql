drop TABLE rovers;

CREATE TABLE rovers (
    rover_id SERIAL PRIMARY KEY,
    initial_id INTEGER NOT NULL,
    rover_status INTEGER NOT NULL,
	user_id INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

-- DROP PROCEDURE public.create_new_rover(in int4, in int4, in int4, out int4);

CREATE OR REPLACE PROCEDURE public.create_new_rover(
    IN initial_id integer,
    IN rover_status integer,
    IN user_id integer,
    OUT rover_id integer
)
LANGUAGE plpgsql
AS $procedure$
BEGIN
    -- Insert the values and store the returned id into the OUT parameter
    INSERT INTO rovers (initial_id, rover_status, user_id)
    VALUES (initial_id, rover_status, user_id)
    RETURNING rovers.rover_id INTO rover_id;
EXCEPTION
    WHEN OTHERS THEN
        -- Log the error and set rover_id to null to indicate failure
        RAISE NOTICE 'Insertion failed: %', SQLERRM;
        rover_id := null;
END;
$procedure$;

DO $$
DECLARE
    rover_id int; -- Declare a variable to capture the OUT parameter
BEGIN
    -- Call the procedure
	CALL create_new_rover(987, 987, 987, rover_id);

    -- Display the result
    RAISE NOTICE 'Generated Result ID: %', rover_id;
    -- output
    -- Generated Result ID: t
END;
$$;

-- insert into rovers values (1234,1,4321,1234);
-- insert into rovers (initial_id,rover_status,user_id) values (1234,1,4321);
