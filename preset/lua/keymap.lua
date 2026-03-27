---@alias Mode "main"|"input"

---@class lc.keymap
local keymap = {}

---@class lc.ConfigKeymap
---@field up? string
---@field down? string
---@field top? string
---@field bottom? string
---@field preview_up? string
---@field preview_down? string
---@field reload? string
---@field quit? string
---@field force_quit? string
---@field filter? string
---@field clear_filter? string
---@field back? string
---@field open? string
---@field enter? string

---Set a key mapping for a specific mode
---@param mode Mode The mode (e.g., "main", "input")
---@param key string The key sequence (e.g., "ctrl-d", "down", "j")
---@param callback string|fun() The command string or callback function
function keymap.set(mode, key, callback) return _lc.keymap.set(mode, key, callback) end

---@alias EntryKeymap table<string, fun()>

lc.keymap = keymap
