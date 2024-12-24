DROP FUNCTION public.get_rover_data(int4);

CREATE OR REPLACE FUNCTION get_rover_data(p_user_id INTEGER)
RETURNS TABLE (
    rover_id INTEGER,
    initial_id INTEGER,
    rover_status INTEGER,
    user_id INTEGER,
    created_at TEXT
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT 
        r.rover_id, 
        r.initial_id, 
        r.rover_status, 
        r.user_id, 
        r.created_at::TEXT
    FROM rovers r
    WHERE r.user_id = p_user_id;
END;
$$;

SELECT * FROM get_rover_data(3);