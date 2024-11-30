-- Drop the existing function (if it exists)
DROP FUNCTION IF EXISTS get_rover_operation_data(TEXT);

-- Create the updated function
CREATE OR REPLACE FUNCTION get_rover_operation_data(
    p_rover_id TEXT -- Input as TEXT
)
RETURNS TABLE(
    id INT,
    rover_id INT,
    random_id INT,
    battery_status DOUBLE PRECISION,
    temp DOUBLE PRECISION,
    humidity DOUBLE PRECISION,
    result_image TEXT,
    image_data TEXT,
    created_at TEXT -- Cast created_at to TEXT if required by the application
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        op.id,
        op.rover_id,
        op.random_id,
        op.battery_status,
        op.temp,
        op.humidity,
        op.result_image,
        op.image_data,
        op.created_at::TEXT -- Explicitly cast to TEXT
    FROM operations op
    WHERE op.rover_id::TEXT = p_rover_id; -- Cast rover_id to TEXT for comparison
END;
$$ LANGUAGE plpgsql;

SELECT * FROM get_rover_operation_data('1');



