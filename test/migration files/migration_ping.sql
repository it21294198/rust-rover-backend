
DROP PROCEDURE public.get_ping(in text, out text);

CREATE OR REPLACE PROCEDURE get_ping(
    ping TEXT,
    OUT pong TEXT
)
LANGUAGE plpgsql
AS $$
BEGIN
    -- Concatenate 'ping' and 'pong' and assign to 'pong'
    pong := ping || 'pong';

EXCEPTION
    WHEN OTHERS THEN
        -- Log the error message
        RAISE NOTICE 'Error: %', SQLERRM;
        -- Optionally, set 'pong' to indicate error or handle as needed
        pong := 'Error';
END;
$$;

-- Declare a variable to store the output
DO $$
DECLARE
    result TEXT;
BEGIN
    -- Call the procedure and store the result in the variable 'result'
    CALL get_ping('ping', result);

    -- Optionally, you can display the result
    RAISE NOTICE 'Result: %', result;
END $$;
