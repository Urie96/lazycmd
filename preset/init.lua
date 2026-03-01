local args = lc.api.argv()
local cfg = {
  default_plugin = args[2],
}

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
  plugin.list(path, function(entries)
    if lc.equals(path, lc.api.get_current_path()) then lc.api.page_set_entries(entries) end
  end)
end

function lc._preview()
  local entry = lc.api.page_get_hovered()
  local path = lc.api.get_hovered_path()
  if entry then
    plugin.preview(entry, function(entries)
      if lc.equals(path, lc.api.get_hovered_path()) then lc.api.page_set_preview(entries) end
    end)
  end
end
