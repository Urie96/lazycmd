--
-- clipboard.lua
--
-- System clipboard access via lc.clipboard.get() and lc.clipboard.set()
-- Implemented in Rust (arboard crate)
--

---@class lc.clipboard
local clipboard = {}

---Get the current clipboard content
---@return string content The clipboard text content
function clipboard.get()
  return _lc.clipboard.get()
end

---Set the clipboard content
---@param text string The text to copy to the clipboard
function clipboard.set(text)
  _lc.clipboard.set(text)
end

lc.clipboard = clipboard
