local cfg = {
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
    help = '?',
    delete = 'dd',
    new = 'n',
    append = 'a',
    input_submit = '<enter>',
    input_cancel = '<esc>',
    input_clear_before_cursor = '<C-u>',
    input_cursor_to_start = '<C-a>',
    input_cursor_to_end = '<C-e>',
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

local runtime = {
  explicit_plugin_specs = {},
  plugin_specs_by_name = {},
  loaded_plugins = {},
  configured_plugins = {},
}

local function rebuild_plugin_index()
  runtime.explicit_plugin_specs = {}
  runtime.plugin_specs_by_name = {}
  runtime.loaded_plugins = {}
  runtime.configured_plugins = {}

  for _, plugin_spec in ipairs(cfg.plugins or {}) do
    local spec = lc._pm.parse_plugin_spec(plugin_spec)
    if spec then
      table.insert(runtime.explicit_plugin_specs, spec)
      runtime.plugin_specs_by_name[spec.name] = spec
    end
  end
end

local function plugin_status(spec)
  if spec.is_remote and not lc._pm.is_installed(spec) then return 'missing' end
  return 'installed'
end

local function plugin_display(spec)
  local color = plugin_status(spec) == 'missing' and 'yellow' or 'white'
  return lc.style.line {
    lc.style.span(spec.name):fg(color),
  }
end

local function list_root_plugins(cb)
  local entries = {}
  for _, spec in ipairs(runtime.explicit_plugin_specs) do
    table.insert(entries, {
      key = spec.name,
      repo = spec.repo,
      url = spec.url,
      dir = spec.dir,
      is_remote = spec.is_remote,
      status = plugin_status(spec),
      display = plugin_display(spec),
      preview = function(self, preview_cb) lc._manager.preview(self, preview_cb) end,
    })
  end
  cb(entries)
end

local function load_plugin(name)
  local spec = runtime.plugin_specs_by_name[name]
  if not spec then return nil, 'Unknown plugin: ' .. tostring(name) end

  if runtime.loaded_plugins[name] == nil then
    local ok, plugin = pcall(require, name)
    if not ok then return nil, plugin end
    runtime.loaded_plugins[name] = plugin or {}
  end

  return runtime.loaded_plugins[name], spec
end

local function ensure_plugin(name)
  local plugin, spec_or_err = load_plugin(name)
  if not plugin then return nil, spec_or_err end
  local spec = spec_or_err

  if not runtime.configured_plugins[name] then
    local ok_config, config_err = pcall(spec.config)
    if not ok_config then return nil, config_err end
    runtime.configured_plugins[name] = true
  end

  return runtime.loaded_plugins[name]
end

local function setup_plugin(name) return ensure_plugin(name) end

local function guarded_preview_callback(hovered_path)
  return function(preview)
    if lc.deep_equal(hovered_path, lc.api.get_hovered_path()) then lc.api.page_set_preview(preview) end
  end
end

local function open_help()
  local options = {}
  local lines = {}

  for _, item in ipairs(lc.api.get_available_keymaps()) do
    local source = item.source == 'entry' and '[entry]' or '[global]'
    local desc = item.desc or 'no description'
    local line = lc.style.line {
      lc.style.span(item.key):fg 'yellow',
      '  ',
      lc.style.span(source):fg(item.source == 'entry' and 'cyan' or 'blue'),
      '  ',
      lc.style.span(desc):fg 'white',
    }
    table.insert(lines, line)
    table.insert(options, {
      value = item,
      display = line,
    })
  end

  lc.style.align_columns(lines)

  lc.select({ prompt = 'Available Keymaps', options = options }, function(choice)
    if choice and choice.callback then choice.callback() end
  end)
end

local function open_filter()
  lc.input {
    prompt = 'Filter:',
    placeholder = '输入筛选内容...',
    value = lc.api.get_filter(),
    on_change = function(input) lc.api.set_filter(input) end,
    on_submit = function(input) lc.api.set_filter(input) end,
    on_cancel = function() lc.api.set_filter '' end,
  }
end

local function apply_configured_keymap()
  local map = function(key, cb, desc) lc.keymap.set('main', key, cb, { desc = desc }) end
  local map_input = function(key, cb, desc) lc.keymap.set('input', key, cb, { desc = desc }) end
  map(cfg.keymap.up, 'scroll_by -1', 'move up')
  map(cfg.keymap.down, 'scroll_by 1', 'move down')
  map(cfg.keymap.top, 'scroll_by -9999', 'go to top')
  map(cfg.keymap.bottom, 'scroll_by 9999', 'go to bottom')
  map(cfg.keymap.preview_up, 'scroll_preview_by -30', 'scroll preview up')
  map(cfg.keymap.preview_down, 'scroll_preview_by 30', 'scroll preview down')
  map(cfg.keymap.reload, 'reload', 'reload')
  map(cfg.keymap.quit, 'quit', 'quit')
  map(cfg.keymap.force_quit, 'quit', 'force quit')
  map(cfg.keymap.filter, open_filter, 'filter')
  map(cfg.keymap.clear_filter, function() lc.api.set_filter '' end, 'clear filter')
  map(cfg.keymap.back, 'back', 'back')
  map(cfg.keymap.open, 'enter', 'open')
  map(cfg.keymap.enter, 'enter', 'enter')
  map(cfg.keymap.help, open_help, 'help')
  map('gr', function() lc.api.go_to {} end, 'go to /')

  map_input(cfg.keymap.input_submit, 'input_submit', 'submit input')
  map_input(cfg.keymap.input_cancel, 'input_cancel', 'cancel input')
  map_input(
    cfg.keymap.input_clear_before_cursor,
    'input_clear_before_cursor',
    'delete text before cursor'
  )
  map_input(
    cfg.keymap.input_cursor_to_start,
    'input_cursor_to_start',
    'move cursor to start'
  )
  map_input(cfg.keymap.input_cursor_to_end, 'input_cursor_to_end', 'move cursor to end')
end

local config = {}

function config.get() return cfg end

function config.setup(opt)
  cfg = lc.tbl_deep_extend('force', cfg, opt or {})
  rebuild_plugin_index()
  add_plugin_paths(cfg.plugins)
  apply_configured_keymap()
  lc._manager.setup(cfg.plugins)
  lc._pm.install_missing(cfg.plugins, function() lc.cmd 'reload' end)

  function lc._list()
    local path = lc.api.get_current_path()
    if #path == 0 then
      list_root_plugins(function(entries)
        if lc.deep_equal(path, lc.api.get_current_path()) then lc.api.page_set_entries(entries) end
      end)
      return
    end

    local plugin, err = ensure_plugin(path[1])
    if not plugin then
      lc.notify(tostring(err))
      lc.api.page_set_entries {}
      return
    end

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

    local current_path = lc.api.get_current_path()
    if #current_path == 0 then
      cb ''
      return
    end

    local plugin = ensure_plugin(current_path[1])
    if plugin and plugin.preview then plugin.preview(entry, cb) end
  end
end

lc.config = config
lc.plugin = lc.plugin or {}

function lc.plugin.load(name) return setup_plugin(name) end

-- Set metatable on lc.system to handle multiple argument formats
setmetatable(lc.config, {
  __call = function(self, opt) lc.config.setup(opt) end,
})

require 'init'
