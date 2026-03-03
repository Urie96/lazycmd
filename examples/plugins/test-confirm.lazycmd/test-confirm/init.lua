local M = {}

function M.setup()
  -- 添加键盘映射来触发确认对话框
  lc.keymap.set('main', 'c', function()
    lc.notify 'Press "c" again to test confirm dialog'
  end)

  lc.keymap.set('main', 'C', function()
    lc.confirm({
      prompt = 'Do you want to delete this item?',
      on_confirm = function()
        lc.notify 'Confirmed! Item would be deleted.'
      end,
      on_cancel = function()
        lc.notify 'Cancelled. No action taken.'
      end
    })
  end)

  -- 添加另一个测试
  lc.keymap.set('main', 'X', function()
    lc.confirm({
      prompt = 'Are you sure you want to exit?',
      on_confirm = function()
        lc.notify 'Exiting...'
        lc.cmd 'quit'
      end,
      on_cancel = function()
        lc.notify 'Exit cancelled.'
      end
    })
  end)
end

function M.list(path, cb)
  cb({
    {key = "1", display = "Test item 1"},
    {key = "2", display = "Test item 2"},
    {key = "3", display = "Test item 3"},
  })
end

function M.preview(entry, cb)
  cb("Preview for: " .. entry.display)
end

return M
