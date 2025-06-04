local tokens_key = KEYS[1]
local last_refill_key = tokens_key .. ':last_refill'
local max_tokens = tonumber(ARGV[1])
local refill_rate = tonumber(ARGV[2])

-- Get current time from Redis
local time_result = redis.call('TIME')
local current_time = tonumber(time_result[1])

-- Get current values
local current_tokens = tonumber(redis.call('GET', tokens_key)) or max_tokens
local last_refill = tonumber(redis.call('GET', last_refill_key)) or current_time

-- Calculate tokens to add based on time elapsed
local time_elapsed = current_time - last_refill
local tokens_to_add = math.floor(time_elapsed * refill_rate)

-- Update token count (cap at max_tokens)
local new_tokens = math.min(current_tokens + tokens_to_add, max_tokens)

-- Check if request can be fulfilled
local allowed = 0
local remaining = new_tokens

if new_tokens > 0 then
    allowed = 1
    remaining = new_tokens - 1
    -- Update Redis with new values (no expiration needed for token buckets)
    redis.call('SET', tokens_key, remaining)
    redis.call('SET', last_refill_key, current_time)
else
    -- Update timestamp even if no tokens available
    redis.call('SET', last_refill_key, current_time)
end

-- Calculate reset time (when bucket will be full again)
local tokens_needed = max_tokens - remaining
local reset_time = current_time + math.ceil(tokens_needed / refill_rate)

return { allowed, remaining, reset_time }
