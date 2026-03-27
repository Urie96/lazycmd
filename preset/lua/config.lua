local args = lc.api.argv()
local cfg = {
  default_plugin = args[2],
  keymap = {
    up = '<up>',
    down = '<down>',
    top = 'gg',
    bottom = 'G',
    preview_up = '<pageup>',
    preview_down = '<pagedown>',
    reload = '<C-r>',
    quit = 'q',
    force_quit = '<C-q>',
    filter = '/',
    clear_filter = '<esc>',
    back = '<left>',
    open = '<right>',
    enter = '<enter>',
  },
}

local function append_package_path(paths, path, seen)
  if path and path ~= '' and not seen[path] then
    seen[path] = true
    table.insert(paths, path)
  end
end

local function add_config_base_path()
  local package = require 'package'
  local base_dir = os.getenv 'HOME' .. '/.config/lazycmd'

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

local function guarded_preview_callback(hovered_path)
  return function(preview)
    if lc.deep_equal(hovered_path, lc.api.get_hovered_path()) then lc.api.page_set_preview(preview) end
  end
end

local function apply_configured_keymap()
  local map = function(key, cb) lc.keymap.set('main', key, cb) end
  map(cfg.keymap.up, 'scroll_by -1')
  map(cfg.keymap.down, 'scroll_by 1')
  map(cfg.keymap.top, 'scroll_by -9999')
  map(cfg.keymap.bottom, 'scroll_by 9999')
  map(cfg.keymap.preview_up, 'scroll_preview_by -30')
  map(cfg.keymap.preview_down, 'scroll_preview_by 30')
  map(cfg.keymap.reload, 'reload')
  map(cfg.keymap.quit, 'quit')
  map(cfg.keymap.force_quit, 'quit')
  map(cfg.keymap.filter, function()
    lc.input {
      prompt = 'Filter:',
      placeholder = '输入筛选内容...',
      value = lc.api.get_filter(),
      on_change = function(input) lc.api.set_filter(input) end,
      on_submit = function(input) lc.api.set_filter(input) end,
      on_cancel = function() lc.api.set_filter '' end,
    }
  end)
  map(cfg.keymap.clear_filter, function() lc.api.set_filter '' end)
  map(cfg.keymap.back, 'back')
  map(cfg.keymap.open, 'enter')
  map(cfg.keymap.enter, 'enter')
end

local config = {}

function config.get() return cfg end

function config.setup(opt)
  cfg = lc.tbl_deep_extend('force', cfg, opt or {})
  add_plugin_paths(cfg.plugins)
  apply_configured_keymap()

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
        if lc.deep_equal(path, lc.api.get_current_path()) then lc.api.page_set_entries(entries) end
      end)
    end
  end

  function lc._preview()
    local entry = lc.api.page_get_hovered()
    local path = lc.api.get_hovered_path()
    if not entry then return end

    local cb = guarded_preview_callback(path)
    if type(entry.preview) == 'function' then
      local preview_text = entry:preview(cb)
      if preview_text then cb(preview_text) end
      return
    end

    if plugin.preview then plugin.preview(entry, cb) end
  end
end

lc.config = config

-- Set metatable on lc.system to handle multiple argument formats
setmetatable(lc.config, {
  __call = function(self, opt) lc.config.setup(opt) end,
})

require 'init'
