---@class lc.url
local url = {}

---Percent-encode a string for safe inclusion in URL components.
---@param value string
---@return string encoded
function url.encode(value)
  return _lc.url.encode(value)
end

---Decode a percent-encoded string.
---@param value string
---@return string decoded
function url.decode(value)
  return _lc.url.decode(value)
end

lc.url = url
