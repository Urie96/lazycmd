---@alias Mode "main"|"input"

---@class lc.keymap
local keymap = {}

---Set a key mapping for a specific mode
---@param mode Mode The mode (e.g., "main", "input")
---@param key string The key sequence (e.g., "ctrl-d", "down", "j")
---@param callback string|fun() The command string or callback function
function keymap.set(mode, key, callback) return _lc.keymap.set(mode, key, callback) end

lc.keymap = keymap
