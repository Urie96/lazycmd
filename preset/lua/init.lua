local args = lc.api.argv()
local cfg = {
  default_plugin = args[2],
}

local function append_package_path(paths, path, seen)
  if path and path ~= '' and not seen[path] then
    seen[path] = true
    table.insert(paths, path)
  end
end

local function add_config_base_path()
  local package = require 'package'
  local base_dir = rawget(_G, '__lazycmd_config_base_dir')
  if not base_dir or base_dir == '' then return end

  local paths = { package.path }
  local seen = {}
  append_package_path(paths, base_dir .. '/?.lua', seen)
  append_package_path(paths, base_dir .. '/?/init.lua', seen)
  package.path = table.concat(paths, ';')
end

local function add_plugin_paths(plugins)
  local package = require 'package'
  local paths = { package.path }
  local seen = {}

  for _, p in ipairs(lc._pm.flatten_plugins(plugins or {})) do
    if p.dir and not seen[p.dir] then
      append_package_path(paths, p.dir .. '/?.lua', seen)
      append_package_path(paths, p.dir .. '/?/init.lua', seen)
    elseif p.is_remote and p.install_path then
      append_package_path(paths, p.install_path .. '/?.lua', seen)
      append_package_path(paths, p.install_path .. '/?/init.lua', seen)
    end
  end

  package.path = table.concat(paths, ';')
end

add_config_base_path()

function lc.config(opt)
  cfg = lc.tbl_extend(cfg, opt or {})
  add_plugin_paths(cfg.plugins)

  local ok, plugin = pcall(require, cfg.default_plugin)

  if not ok then
    lc._manager.setup(cfg.plugins)
    lc._pm.install_missing(cfg.plugins, function() lc.cmd 'reload' end)
    plugin = lc._manager
  else
    -- When launching a plugin directly, still apply its configured setup.
    for _, p in ipairs(cfg.plugins or {}) do
      local spec = lc._pm.parse_plugin_spec(p)
      if spec and spec.name == cfg.default_plugin then
        spec.config()
        break
      end
    end
  end

  function lc._list()
    local path = lc.api.get_current_path()
    if plugin.list then
      plugin.list(path, function(entries)
        if lc.equals(path, lc.api.get_current_path()) then lc.api.page_set_entries(entries) end
      end)
    end
  end

  function lc._preview()
    local entry = lc.api.page_get_hovered()
    local path = lc.api.get_hovered_path()
    if entry and plugin.preview then
      plugin.preview(entry, function(entries)
        if lc.equals(path, lc.api.get_hovered_path()) then lc.api.page_set_preview(entries) end
      end)
    end
  end
end

require 'init'
