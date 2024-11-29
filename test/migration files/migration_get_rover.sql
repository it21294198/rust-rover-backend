
DROP PROCEDURE public.get_rover(in int4, out text);

CREATE OR REPLACE PROCEDURE get_rover(
    initial_rover_id INTEGER,
    OUT rover_status TEXT
)
LANGUAGE plpgsql
AS $$
BEGIN
    -- Select the rover_status from the rovers table based on initial_rover_id
    SELECT r.rover_status::TEXT
    INTO rover_status
    FROM rovers r
    WHERE r.initial_id = initial_rover_id  -- Corrected: Changed initial_id to rover_id
    LIMIT 1;

    -- If no row is found, set rover_status to 'Not Found'
    IF NOT FOUND THEN
        rover_status := 'Not Found';
    END IF;

EXCEPTION
    WHEN OTHERS THEN
        -- Log the error and set the rover_status to 'Error'
        RAISE NOTICE 'Error: %', SQLERRM;
        rover_status := 'Error';
END;
$$;

DO $$
DECLARE
    rover_status TEXT;
BEGIN
    -- Call the procedure and capture the output
    CALL get_rover(1, rover_status);

    -- Output the result
    RAISE NOTICE 'Rover Status: %', rover_status;
END $$;