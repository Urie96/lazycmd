--
-- yaml.lua
--
-- YAML encoding and decoding via lc.yaml.decode() and lc.yaml.encode()
-- Implemented in Rust (serde_yaml)
--

---@class lc.yaml
local yaml = {}

---Decode a YAML string to a Lua value
---@param str string The YAML string to decode
---@return any lua_value The decoded Lua value
function yaml.decode(str)
  return _lc.yaml.decode(str)
end

---Encode a Lua value to a YAML string
---@param val any The Lua value to encode (nil, boolean, number, string, table, array)
---@return string yaml_string The YAML encoded string
function yaml.encode(val)
  return _lc.yaml.encode(val)
end

lc.yaml = yaml
