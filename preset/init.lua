package.path = package.path .. ";./preset/?.lua"

require("inject")
local inspect = require("inspect")

string.__add = function(a, b)
  return a .. b
end

local M = {}

local lc = lc or {}
lc.print = function(...)
  print(lc.inspect(...))
end
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

map("main", "up", "scroll_by -1")
map("main", "down", "scroll_by 1")
map("main", "pageup", "scroll_preview_by -30")
map("main", "pagedown", "scroll_preview_by 30")
map("main", "q", "quit")
map("main", "gh", function()
  lc.api.go_to(lc.path.split(os.getenv("HOME")))
end)
map("main", "gd", function()
  lc.api.go_to(lc.path.split(os.getenv("PWD")))
end)
map("main", "left", function()
  local path = lc.api.get_current_path()
  if #path > 0 then
    table.remove(path)
    lc.api.go_to(path)
  end
end)

map("main", "right", function()
  local hovered = lc.api.page_get_hovered()
  if hovered and hovered.is_dir then
    local path = lc.api.get_current_path()
    table.insert(path, hovered.key)
    lc.api.go_to(path)
  end
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
  local hovered = lc.api.page_get_hovered()
  if hovered then
    table.insert(path, hovered.key)
    if hovered.is_dir == false then
      local filepath = lc.path.join(path)
      lc.system({ "bat", "-p", "--color=always", filepath }, nil, function(out)
        if out.code == 0 then
          lc.api.page_set_preview(path, out.stdout:ansi())
        else
          lc.api.page_set_preview(path, out.stderr:ansi())
        end
      end)
    elseif hovered.is_dir then
      local hovered_dir_path = lc.path.join(path)
      local filenames = lc.tbl_map(function(e)
        return e.name
      end, lc.fs.read_dir_sync(hovered_dir_path))
      lc.api.page_set_preview(path, table.concat(filenames, "\n"))
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
