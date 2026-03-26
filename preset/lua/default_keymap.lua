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
map('main', '/', function()
  lc.input {
    prompt = 'Filter:',
    placeholder = '输入筛选内容...',
    value = lc.api.get_filter(),
    on_change = function(input) lc.api.set_filter(input) end,
    on_submit = function(input) lc.api.set_filter(input) end,
    on_cancel = function() lc.api.set_filter '' end,
  }
end)
map('main', '<esc>', function()
  -- Clear filter when pressing Escape
  lc.api.set_filter ''
end)
map('main', '<left>', 'back')
map('main', '<right>', 'enter')
map('main', '<enter>', 'enter')
