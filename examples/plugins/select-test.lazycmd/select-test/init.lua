local M = {}

function M.setup()
  -- Test 0: Text notification with styling
  lc.keymap.set('main', 't', function()
    local text = lc.style.text({
      lc.style.span('✓'):fg('green'),
      lc.style.span(' 成功! '):fg('white'),
      lc.style.span('操作完成'):fg('blue'),
    })
    lc.notify(text)
  end)

  -- Test 1: Line notification with styling
  lc.keymap.set('main', 'l', function()
    local line = lc.style.line({
      lc.style.span('⚠ '):fg('yellow'),
      lc.style.span('警告信息'):fg('yellow'),
    })
    lc.notify(line)
  end)

  -- Test 2: Span notification with styling
  lc.keymap.set('main', 's', function()
    local span = lc.style.span('✕ 操作失败'):fg('red')
    lc.notify(span)
  end)

  -- Test 3: Simple string options (select dialog)
  lc.keymap.set('main', 'S', function()
    lc.select({
      prompt = '请选择一个选项:',
      options = { '选项1', '选项2', '选项3' },
    }, function(choice)
      if choice then
        lc.notify('选择了: ' .. choice)
      else
        lc.notify '取消了选择'
      end
    end)
  end)

  -- Test 4: Options with value and display (select dialog)
  lc.keymap.set('main', 'p', function()
    lc.select({
      prompt = '选择一种编程语言:',
      options = {
        { value = 'py', display = '🐍 Python' },
        { value = 'js', display = '📜 JavaScript' },
        { value = 'lua', display = '🌙 Lua' },
        { value = 'rs', display = '🦀 Rust' },
      },
    }, function(choice)
      if choice then
        lc.notify('选择了: ' .. choice)
      else
        lc.notify '取消了选择'
      end
    end)
  end)

  -- Test 5: Long list with filtering (select dialog)
  lc.keymap.set('main', 'L', function()
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
        lc.notify '取消了选择'
      end
    end)
  end)

  -- Initialize with some dummy entries
  lc.api.page_set_entries {
    { key = 'test0', display = "按 't' 测试 Text 通知" },
    { key = 'test1', display = "按 'l' 测试 Line 通知" },
    { key = 'test2', display = "按 's' 测试 Span 通知" },
    { key = 'test3', display = "按 'S' 测试选择对话框" },
    { key = 'test4', display = "按 'p' 测试带图标选择" },
    { key = 'test5', display = "按 'L' 测试长列表筛选" },
  }
end

function M.list(path, cb)
  cb {
    { key = 'test0', display = "按 't' 测试 Text 通知" },
    { key = 'test1', display = "按 'l' 测试 Line 通知" },
    { key = 'test2', display = "按 's' 测试 Span 通知" },
    { key = 'test3', display = "按 'S' 测试选择对话框" },
    { key = 'test4', display = "按 'p' 测试带图标选择" },
    { key = 'test5', display = "按 'L' 测试长列表筛选" },
  }
end

return M
