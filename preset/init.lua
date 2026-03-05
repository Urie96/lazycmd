local args = lc.api.argv()
local cfg = {
  default_plugin = args[2],
}

-- Redefine lc.interactive to handle multiple argument formats
lc.system.interactive = function(cmd, opts_or_callback, callback)
  -- Parse arguments:
  -- lc.interactive(cmd)
  -- lc.interactive(cmd, callback)
  -- lc.interactive(cmd, opts)
  -- lc.interactive(cmd, opts, callback)

  local args_table = {
    cmd = cmd,
  }

  if type(opts_or_callback) == 'function' then
    -- lc.interactive(cmd, callback)
    args_table.on_complete = opts_or_callback
  elseif type(opts_or_callback) == 'table' then
    -- lc.interactive(cmd, opts) or lc.interactive(cmd, opts, callback)
    if type(opts_or_callback.wait_confirm) == 'function' then
      -- wait_confirm 是一个函数
      args_table.wait_confirm = opts_or_callback.wait_confirm
    elseif opts_or_callback.wait_confirm == true then
      -- wait_confirm 为 true，创建一个总是返回 true 的函数
      args_table.wait_confirm = function() return true end
    end
    if type(callback) == 'function' then
      args_table.on_complete = callback
    elseif opts_or_callback.on_complete ~= nil then
      args_table.on_complete = opts_or_callback.on_complete
    end
  end

  -- Call the Rust implementation with the table
  lc.system._interactive(args_table)
end

lc.interactive = lc.system.interactive

-- Wrap lc.system._exec to handle multiple argument formats
lc.system.exec = function(cmd_or_args, opts_or_callback, callback)
  -- Parse arguments:
  -- lc.system.exec({cmd, callback})
  -- lc.system.exec(cmd, callback)
  -- lc.system.exec(cmd, opts, callback)

  local args_table = {}

  if type(cmd_or_args) == 'table' then
    -- lc.system.exec({cmd, callback}) or lc.system.exec(args)
    if cmd_or_args.cmd ~= nil then
      -- lc.system.exec({cmd = {...}, callback = ...})
      args_table = cmd_or_args
    else
      -- lc.system.exec({cmd, callback})
      args_table.cmd = cmd_or_args
      args_table.callback = opts_or_callback
    end
  else
    -- lc.system.exec(cmd, callback) or lc.system.exec(cmd, opts, callback)
    args_table.cmd = cmd_or_args

    if type(opts_or_callback) == 'function' then
      -- lc.system.exec(cmd, callback)
      args_table.callback = opts_or_callback
    elseif type(opts_or_callback) == 'table' then
      -- lc.system.exec(cmd, opts, callback)
      if opts_or_callback.stdin ~= nil then args_table.stdin = opts_or_callback.stdin end
      if opts_or_callback.env ~= nil then args_table.env = opts_or_callback.env end
      if type(callback) == 'function' then
        args_table.callback = callback
      elseif opts_or_callback.callback ~= nil then
        args_table.callback = opts_or_callback.callback
      else
        error 'Callback function is required when providing options'
      end
    else
      error 'Callback function is required'
    end
  end

  -- Call the Rust implementation
  lc.system._exec(args_table)
end

-- Set metatable on lc.system to handle multiple argument formats
setmetatable(lc.system, {
  __call = lc.system.exec,
})

function lc.config(opt)
  cfg = lc.tbl_extend(cfg, opt or {})

  if not package.loaded[cfg.default_plugin] then
    if cfg.plugins then
      for _, plugin in ipairs(cfg.plugins) do
        if plugin[1] == cfg.default_plugin then
          if plugin.config then plugin.config() end
          break
        end
      end
    end
  end
end

local function map(mode, key, cb) lc.keymap.set(mode, key, cb) end

map('main', '<up>', 'scroll_by -1')
map('main', '<down>', 'scroll_by 1')
map('main', 'gg', 'scroll_by -9999')
map('main', 'G', 'scroll_by 9999')
map('main', '<pageup>', 'scroll_preview_by -30')
map('main', '<pagedown>', 'scroll_preview_by 30')
map('main', '<C-r>', 'reload')
map('main', 'q', 'quit')
map('main', '<C-q>', 'quit')
map('main', '/', 'enter_filter_mode')
map('main', '<esc>', 'filter_clear')
map('main', '<left>', 'back')
map('main', '<right>', 'enter')

-- Input mode keymaps
map('input', '<esc>', 'exit_filter_mode')
map('input', '<enter>', 'accept_filter')
map('input', '<C-u>', 'filter_clear')

require 'init'

local plugin = require(cfg.default_plugin)

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
