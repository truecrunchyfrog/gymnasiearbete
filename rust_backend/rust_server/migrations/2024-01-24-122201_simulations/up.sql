-- Create ENUM type for result
CREATE TYPE simulation_result AS ENUM('Passed', 'Failed', 'Error');

-- Create simulations table
CREATE TABLE simulations (
    simulation_id SERIAL PRIMARY KEY, ran_at TIMESTAMP NOT NULL, ran_file_id INT REFERENCES files (file_id) NOT NULL, logs TEXT, result simulation_result, time_taken INTERVAL, cpu_time INTERVAL, max_memory_usage INT -- Assuming memory usage is measured in some unit like kilobytes
);

-- Add an index on the ran_at column for performance optimization
CREATE INDEX idx_simulations_ran_at ON simulations (ran_at);