package.path = package.path .. ";./preset/?.lua"

require("inject")
local inspect = require("inspect")

local M = {}

local lc = lc or {}
lc.inspect = inspect

local function map(mode, key, cb)
  if type(cb) == "string" then
    lc.keymap.set(mode, key, function()
      lc.cmd(cb)
    end)
  elseif type(cb) == "function" then
    lc.keymap.set(mode, key, cb)
  end
end

map("main", "up", "scroll_up 1")
map("main", "down", "scroll_down 1")
map("main", "q", "quit")
map("main", "gh", function()
  lc.api.go_to(os.getenv("HOME"):split("/"))
end)
map("main", "gd", function()
  lc.api.go_to(os.getenv("PWD"):split("/"))
end)

lc.on_event("EnterPost", function()
  local path = lc.api.get_current_path()
  local entries = lc.tbl_map(function(e)
    return {
      key = e.name,
      display = e.name,
      is_dir = e.is_dir,
    }
  end, lc.fs.read_dir_sync("/" .. table.concat(path, "/")))

  lc.api.page_set_entries(path, entries)
end)

lc.on_event("HoverPost", function()
  local path = lc.api.get_current_path()
  local entry = lc.api.page_get_hovered()
  if entry then
    table.insert(path, entry.key)
    if entry.is_dir == false then
      local filepath = "/" .. table.concat(path, "/")
      lc.system({ "bat", "-p", "--color=always", filepath }, nil, function(out)
        if out.code == 0 then
          lc.api.page_set_preview(path, out.stdout)
        else
          lc.api.page_set_preview(path, out.stderr)
        end
      end)
    end
  end
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
