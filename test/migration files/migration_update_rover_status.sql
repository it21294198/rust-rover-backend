CREATE OR REPLACE PROCEDURE update_rover_status(
    p_initial_id INTEGER,
    p_rover_status INTEGER,
    p_user_id INTEGER,
    OUT status TEXT
)
LANGUAGE plpgsql
AS $$
BEGIN
    -- Check if a record with the given initial_id already exists
    IF EXISTS (SELECT 1 FROM rovers WHERE user_id = p_user_id) THEN
        -- Update the existing record
        UPDATE rovers
        SET rover_status = p_rover_status, initial_id = p_initial_id, created_at = NOW()
        WHERE user_id = p_user_id;

        -- Set the output status to success
        status := '1';
    ELSE
        -- Set the output status to success
        status := '0';
    END IF;
EXCEPTION
    WHEN OTHERS THEN
        -- In case of any exception, set the output status to failure
        status := '0';
END;
$$;

-- usage

DO $$
DECLARE
    result TEXT;
BEGIN
    CALL update_rover_status(1, 1, 3, result);
    RAISE NOTICE 'Result: %', result;
END;
$$;

-- Verify the table contents
SELECT * FROM rovers;
