package.path = package.path .. ';./preset/?.lua;./preset/plugins/?.lazycmd/?/init.lua;'

require 'inject'
local inspect = require 'inspect'
lc.json = require 'json'
local plugins = require 'plugins'

---@param o1 any|table First object to compare
---@param o2 any|table Second object to compare
---@param ignore_mt boolean|nil True to ignore metatables (a recursive function to tests tables inside tables)
local function equals(o1, o2, ignore_mt)
  if o1 == o2 then return true end
  local o1Type = type(o1)
  local o2Type = type(o2)
  if o1Type ~= o2Type then return false end
  if o1Type ~= 'table' then return false end

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
    if value2 == nil or equals(value1, value2, ignore_mt) == false then return false end
    keySet[key1] = true
  end

  for key2, _ in pairs(o2) do
    if not keySet[key2] then return false end
  end
  return true
end

local M = {}

local lc = lc or {}
lc.inspect = inspect

local function map(mode, key, cb)
  if type(cb) == 'string' then
    lc.keymap.set(mode, key, function() lc.cmd(cb) end)
  elseif type(cb) == 'function' then
    lc.keymap.set(mode, key, cb)
  end
end

map('main', '<up>', 'scroll_by -1')
map('main', '<down>', 'scroll_by 1')
map('main', 'g', 'scroll_by 9999')
-- map('main', '<C-d>', 'scroll_by 1')
map('main', '<pageup>', 'scroll_preview_by -30')
map('main', '<pagedown>', 'scroll_preview_by 30')
map('main', '<C-r>', 'reload')
map('main', 'q', 'quit')
map('main', '<C-q>', 'quit')
map('main', '<left>', function()
  local path = lc.api.get_current_path()
  if #path > 0 then
    table.remove(path)
    lc.api.go_to(path)
  end
end)

map('main', '<right>', function()
  local hovered = lc.api.page_get_hovered()
  if hovered then
    local path = lc.api.get_current_path()
    table.insert(path, hovered.key)
    lc.api.go_to(path)
  end
end)

local args = lc.api.argv()
local plugin_name = args[2]

for _, plugin in ipairs(plugins) do
  if plugin[1] == plugin_name then
    if plugin.config then plugin.config() end
    break
  end
end

local plugin = require(plugin_name)

lc.on_event('EnterPost', function()
  local path = lc.api.get_current_path()
  plugin.list(path, function(entries)
    if equals(path, lc.api.get_current_path()) then lc.api.page_set_entries(entries) end
  end)
end)

lc.on_event('HoverPost', function()
  local entry = lc.api.page_get_hovered()
  local path = lc.api.get_hovered_path()
  if entry then
    plugin.preview(entry, function(entries)
      if equals(path, lc.api.get_hovered_path()) then lc.api.page_set_preview(entries) end
    end)
  end
end)

return M
