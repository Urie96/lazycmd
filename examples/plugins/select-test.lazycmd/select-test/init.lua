local M = {}

function M.setup()
  -- Test 1: Simple string options
  lc.keymap.set('main', 's', function()
    lc.select({
      prompt = '请选择一个选项:',
      options = { '选项1', '选项2', '选项3' },
    }, function(choice)
      if choice then
        lc.notify('选择了: ' .. choice)
      else
        lc.notify('取消了选择')
      end
    end)
  end)

  -- Test 2: Options with value and display
  lc.keymap.set('main', 'p', function()
    lc.select({
      prompt = '选择一种编程语言:',
      options = {
        { value = "py", display = "🐍 Python" },
        { value = "js", display = "📜 JavaScript" },
        { value = "lua", display = "🌙 Lua" },
        { value = "rs", display = "🦀 Rust" },
      },
    }, function(choice)
      if choice then
        lc.notify('选择了: ' .. choice)
      else
        lc.notify('取消了选择')
      end
    end)
  end)

  -- Test 3: Long list with filtering
  lc.keymap.set('main', 'l', function()
    local options = {}
    for i = 1, 50 do
      options[i] = '选项 ' .. i
    end

    lc.select({
      prompt = '选择一个选项 (可以输入筛选):',
      options = options,
    }, function(choice)
      if choice then
        lc.notify('选择了: ' .. choice)
      else
        lc.notify('取消了选择')
      end
    end)
  end)

  -- Initialize with some dummy entries
  lc.api.page_set_entries({
    { key = "test1", display = "按 's' 测试简单选择" },
    { key = "test2", display = "按 'p' 测试带图标的选择" },
    { key = "test3", display = "按 'l' 测试长列表筛选" },
  })
end

function M.list(path, cb)
  cb {
    { key = "test1", display = "按 's' 测试简单选择" },
    { key = "test2", display = "按 'p' 测试带图标的选择" },
    { key = "test3", display = "按 'l' 测试长列表筛选" },
  }
end

return M
