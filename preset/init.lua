local M = {}

lc.keymap.set("up", function()
  print("up")
end)
lc.keymap.set("q", function()
  lc.cmd("quit")
end)

function M:list(path)
  lc.defer_fn(function()
    print("1000ms done")
  end, 1000)
  table.insert(path, "aaaa")
  return path
end

function M:preview() end

return M
