package.path = package.path .. ";./preset/?.lua"

require("inject")
local inspect = require("inspect")

---@param o1 any|table First object to compare
---@param o2 any|table Second object to compare
---@param ignore_mt boolean|nil True to ignore metatables (a recursive function to tests tables inside tables)
local function equals(o1, o2, ignore_mt)
  if o1 == o2 then
    return true
  end
  local o1Type = type(o1)
  local o2Type = type(o2)
  if o1Type ~= o2Type then
    return false
  end
  if o1Type ~= "table" then
    return false
  end

  if not ignore_mt then
    local mt1 = getmetatable(o1)
    if mt1 and mt1.__eq then
      --compare using built in method
      return o1 == o2
    end
  end

  local keySet = {}

  for key1, value1 in pairs(o1) do
    local value2 = o2[key1]
    if value2 == nil or equals(value1, value2, ignore_mt) == false then
      return false
    end
    keySet[key1] = true
  end

  for key2, _ in pairs(o2) do
    if not keySet[key2] then
      return false
    end
  end
  return true
end

local M = {}

local lc = lc or {}
lc.notify = function(...)
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
  local files, err = lc.fs.read_dir_sync(lc.path.join(path))
  if err then
    lc.notify(err)
    return
  end
  local entries = lc.tbl_map(function(e)
    local display = e.name
    if e.is_dir then
      display = e.name:fg("blue")
    end
    return {
      key = e.name,
      is_dir = e.is_dir,
      display = display,
    }
  end, files)

  lc.api.page_set_entries(entries)
end)

lc.on_event("HoverPost", function(path)
  local path = lc.api.get_hovered_path()
  local hovered = lc.api.page_get_hovered()
  if hovered then
    if hovered.is_dir == false then
      lc.system({ "bat", "-p", "--color=always", lc.path.join(path) }, function(out)
        local preview
        if out.code == 0 then
          preview = out.stdout:ansi()
        else
          preview = out.stderr:ansi()
        end
        if equals(path, lc.api.get_hovered_path()) then
          lc.api.page_set_preview(preview)
        end
      end)
    elseif hovered.is_dir then
      local hovered_dir_path = lc.path.join(path)
      local files, err = lc.fs.read_dir_sync(hovered_dir_path)
      if err then
        lc.notify(err)
        return
      end
      local filenames = lc.tbl_map(function(e)
        return e.name
      end, files)
      lc.api.page_set_preview(table.concat(filenames, "\n"))
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
