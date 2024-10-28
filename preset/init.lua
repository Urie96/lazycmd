package.path = package.path .. ";./preset/?.lua"

require("inject")

local M = {}

local lc = lc or {}

local map = lc.keymap.set

map("up", "scroll_up 1")
map("down", "scroll_down 1")
map("q", "quit")
map("gh", function()
  lc.api.go_to(os.getenv("HOME"):split("/"))
end)

lc.on_event("enter", function()
  local path = lc.api.get_current_path()
  local entries = lc.tbl_map(function(e)
    return {
      key = e.name,
      display = e.name,
    }
  end, lc.fs.read_dir_sync("/" .. table.concat(path, "/")))

  lc.api.page_set_entries(entries)
end)

function M:list(path)
  -- lc.defer_fn(function()
  --   print("1000ms done")
  -- end, 1000)
  table.insert(path, "aaaa")
  return path
end

function M:preview() end

return M
