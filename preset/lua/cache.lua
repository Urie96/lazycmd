---@class CacheOptions
---@field ttl number? Time-to-live in seconds (optional)

---@class lc.cache
local cache = {}

---Get a value from cache
---@param key string The cache key
---@return any value The cached value, or nil if not found or expired
function cache.get(key) return _lc.cache.get(key) end

---Set a value in cache
---@param key string The cache key
---@param value any The value to cache (nil, boolean, number, string, table, array)
---@param opts CacheOptions? Optional options (e.g., {ttl = 3600} for 1 hour TTL)
function cache.set(key, value, opts) return _lc.cache.set(key, value, opts) end

---Delete a value from cache
---@param key string The cache key to delete
function cache.delete(key) return _lc.cache.delete(key) end

---Clear all cached values
function cache.clear() return _lc.cache.clear() end

lc.cache = cache
