unpack = unpack or table.unpack

---Copy text to system clipboard using OSC 52 escape sequence
---@param text string The text to copy
function lc.osc52_copy(text) return _lc.osc52_copy(text) end
