local M = {}

local lc = lc or {}

function lc.tbl_map(func, t)
  local rettab = {} --- @type table<any,any>
  for k, v in pairs(t) do
    rettab[k] = func(v)
  end
  return rettab
end

local map = lc.keymap.set

map("up", "scroll_up 1")
map("down", "scroll_down 1")
map("q", "quit")

lc.on_event("enter", function()
  local path = lc.api.get_current_path()
  if #path > 0 then
    local entries = lc.tbl_map(function(e)
      return {
        key = e.name,
        display = e.name,
      }
    end, lc.fs.read_dir_sync("/" .. table.concat(path, "/")))

    lc.api.page_set_entries(entries)
  end
end)

lc.api.go_to({ "Users", "bytedance" })

function M:list(path)
  -- lc.defer_fn(function()
  --   print("1000ms done")
  -- end, 1000)
  table.insert(path, "aaaa")
  return path
end

function M:preview() end

return M
