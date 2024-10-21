local M = {}

local map = lc.keymap.set

map("up", "scroll_up 1")
map("down", "scroll_down 1")
map("q", "quit")

function M:list(path)
  -- lc.defer_fn(function()
  --   print("1000ms done")
  -- end, 1000)
  table.insert(path, "aaaa")
  return path
end

function M:preview() end

return M
