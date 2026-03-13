--
-- base64.lua
--
-- Base64 encoding and decoding via lc.base64.decode() and lc.base64.encode()
-- Implemented in Rust (base64 crate)
--

---@class lc.base64
local base64 = {}

---Decode a base64 string to a Lua string
---@param encoded string The base64 encoded string
---@return string decoded The decoded string (raw bytes)
function base64.decode(encoded)
  return _lc.base64.decode(encoded)
end

---Encode a Lua string to base64
---@param data string The string to encode
---@return string encoded The base64 encoded string
function base64.encode(data)
  return _lc.base64.encode(data)
end

lc.base64 = base64
