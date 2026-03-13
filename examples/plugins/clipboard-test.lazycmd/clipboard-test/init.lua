-- Clipboard Test Plugin
-- Tests lc.clipboard.get() and lc.clipboard.set() functions

local M = {}

function M.list(path, cb)
  -- Get current clipboard content
  local content = lc.clipboard.get()

  local entries = {
    { key = 'view', display = ('Current clipboard content:'):fg('green') },
    { key = 'content', display = ('  ' .. (content or '(empty)')):fg('gray') },
    { key = 'spacer', display = ' ' },
    { key = 'actions', display = ('Actions:'):fg('cyan') },
    { key = 'copy', display = ('  [Enter] Copy test text to clipboard'):fg('yellow') },
    { key = 'refresh', display = ('  [r] Refresh clipboard content'):fg('yellow') },
    { key = 'test', display = ('  [Ctrl+r] Test copy to clipboard'):fg('yellow') },
  }

  cb(entries)
end

function M.preview(entry, cb)
  local preview

  if entry.key == 'copy' then
    local test_text = 'Hello from lazycmd clipboard API! ' .. os.date('%Y-%m-%d %H:%M:%S')
    lc.clipboard.set(test_text)

    preview = {
      ('Text copied to clipboard!'):fg('green'),
      ' ',
      ('Copied text:'):fg('cyan'),
      (test_text):fg('yellow'),
    }
  elseif entry.key == 'refresh' then
    lc._list()
    return
  elseif entry.key == 'test' then
    local test_text = 'Hello from lazycmd clipboard API! ' .. os.date('%Y-%m-%d %H:%M:%S')
    lc.clipboard.set(test_text)

    preview = {
      ('Text copied to clipboard!'):fg('green'),
      ' ',
      ('Copied text:'):fg('cyan'),
      (test_text):fg('yellow'),
    }
  elseif entry.key == 'content' then
    local content = lc.clipboard.get()
    preview = {
      ('Clipboard content:'):fg('cyan'),
      ' ',
      (content or '(empty)'):fg('yellow'),
    }
  else
    preview = {
      ('Clipboard Test Plugin'):fg('green'),
      ' ',
      ('This plugin demonstrates the lc.clipboard API:'):fg('cyan'),
      ' ',
      ('  lc.clipboard.get() - Get clipboard content'):fg('yellow'),
      ('  lc.clipboard.set(text) - Set clipboard content'):fg('yellow'),
    }
  end

  cb(lc.style.text(preview))
end

function M.setup()
  lc.keymap.set('main', 'r', function()
    lc._list()
    lc.notify('Clipboard refreshed')
  end)

  lc.keymap.set('main', '<C-r>', function()
    local test_text = 'Hello from lazycmd clipboard API! ' .. os.date('%Y-%m-%d %H:%M:%S')
    lc.clipboard.set(test_text)
    lc.notify('Copied to clipboard: ' .. test_text)
    lc._list()
  end)
end

return M
