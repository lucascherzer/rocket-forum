-- Redis Lua Script for Token Bucket Rate Limiting
-- KEYS[1]: unique identifier for the rate limit (e.g., user:123 or ip:192.168.1.1)
-- ARGV[1]: max_tokens (bucket capacity)
-- ARGV[2]: refill_rate (tokens per second)
-- ARGV[3]: ttl_seconds (optional, defaults to 3600 seconds)

local tokens_key = KEYS[1]
local last_refill_key = tokens_key .. ':last_refill'
local max_tokens = tonumber(ARGV[1])
local refill_rate = tonumber(ARGV[2]) / 60    -- tokens per minute
local ttl_seconds = tonumber(ARGV[3]) or 3600 -- default 1 hour TTL

-- Input validation
if max_tokens <= 0 or refill_rate <= 0 then
    return redis.error_reply("Invalid parameters: max_tokens and refill_rate must be positive")
end

-- Get current time from Redis (seconds since epoch, microseconds)
local time_result = redis.call('TIME')
local current_time = tonumber(time_result[1]) + (tonumber(time_result[2]) / 1000000)

-- Get current values
local current_tokens = tonumber(redis.call('GET', tokens_key))
local last_refill = tonumber(redis.call('GET', last_refill_key))

-- Initialize if keys don't exist (first request)
if not current_tokens or not last_refill then
    current_tokens = max_tokens
    last_refill = current_time
end

-- Calculate tokens to add based on time elapsed
local time_elapsed = math.max(0, current_time - last_refill)
local tokens_to_add = time_elapsed * refill_rate

-- Update token count (cap at max_tokens)
local new_tokens = math.min(current_tokens + tokens_to_add, max_tokens)

-- Check if request can be fulfilled
local allowed = 0
local remaining = new_tokens

if new_tokens >= 1 then
    allowed = 1
    remaining = new_tokens - 1
end

-- Always update both keys to maintain consistency
redis.call('SETEX', tokens_key, ttl_seconds, remaining)
redis.call('SETEX', last_refill_key, ttl_seconds, current_time)

-- Calculate reset time (when bucket will be full again)
local tokens_needed_for_full = max_tokens - remaining
local reset_time_offset = 0
if tokens_needed_for_full > 0 and refill_rate > 0 then
    reset_time_offset = math.ceil(tokens_needed_for_full / refill_rate)
end
local reset_time = current_time + reset_time_offset

-- Return results
-- allowed: 1 if request is allowed, 0 if rate limited
-- remaining: number of tokens remaining after this request
-- reset_time: timestamp when bucket will be full (for client retry logic)
return {
    allowed,
    math.floor(remaining),
    math.floor(reset_time)
}
