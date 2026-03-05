local M = {}

function M.setup()
  -- 测试新的 wait_confirm 功能
  lc.keymap.set('main', 't', function()
    -- 不带 wait_confirm 的调用
    lc.interactive({'echo', 'Hello from lazycmd!'}, function(exit_code)
      lc.notify('Command exited with code: ' .. exit_code)
    end)
  end)

  -- 测试带 wait_confirm = true 的调用
  lc.keymap.set('main', 'T', function()
    -- 带 wait_confirm = true 的调用
    lc.interactive({'ls', '-la'}, {wait_confirm = true}, function(exit_code)
      lc.notify('Command exited with code: ' .. exit_code)
    end)
  end)

  -- 测试 wait_confirm 为函数：只在出错时等待
  lc.keymap.set('main', 'f', function()
    lc.interactive({'cat', '/nonexistent'}, {
      wait_confirm = function(code)
        return code ~= 0
      end
    }, function(exit_code)
      lc.notify('Command exited with code: ' .. exit_code)
    end)
  end)

  -- 测试 wait_confirm 函数：只在 exit code > 1 时等待
  lc.keymap.set('main', 'F', function()
    lc.interactive({'false'}, {
      wait_confirm = function(code)
        return code > 1
      end
    }, function(exit_code)
      lc.notify('Command exited with code: ' .. exit_code)
    end)
  end)

  -- 测试没有回调的调用
  lc.keymap.set('main', 'e', function()
    lc.interactive({'echo', 'No callback - immediate return'})
  end)

  -- 测试带 wait_confirm = true 但没有回调的调用
  lc.keymap.set('main', 'E', function()
    lc.interactive({'ls'}, {wait_confirm = true})
  end)
end

function M.list(_, cb)
  -- 返回一些测试条目
  local entries = {
    {
      key = '1',
      display = 'Test 1: Echo (t) - Immediate return after completion',
    },
    {
      key = '2',
      display = 'Test 2: ls -la (T) - Always wait for Enter',
    },
    {
      key = '3',
      display = 'Test 3: cat nonexistent (f) - Wait only on error (code ~= 0)',
    },
    {
      key = '4',
      display = 'Test 4: false command (F) - Wait only on severe error (code > 1)',
    },
    {
      key = '5',
      display = 'Test 5: Echo (e) - No callback, immediate return',
    },
    {
      key = '6',
      display = 'Test 6: ls (E) - Always wait, no callback',
    },
  }
  cb(entries)
end

function M.preview(entry, cb)
  local help = [[
这是一个测试 lc.interactive() 新功能的插件

新的 API 支持传入 options 表：
  lc.interactive(cmd, opts, callback)

Options:
  wait_confirm - 可以是以下三种值：
                 1. boolean true: 总是等待用户按 Enter
                 2. boolean false/nil: 从不等待（默认）
                 3. function(code): 根据退出码决定是否等待

使用示例：

1. 按 t 键：执行 echo，立即返回
   lc.interactive({'echo', 'Hello'}, callback)

2. 按 T 键：执行 ls -la，总是等待按 Enter
   lc.interactive({'ls', '-la'}, {wait_confirm = true}, callback)

3. 按 f 键：执行 cat 不存在的文件，只在出错时等待
   lc.interactive({'cat', '/nonexistent'}, {
     wait_confirm = function(code)
       return code ~= 0  -- 退出码非0时等待
     end
   }, callback)

4. 按 F 键：执行 false，只在严重错误时等待（exit code > 1）
   lc.interactive({'false'}, {
     wait_confirm = function(code)
       return code > 1  -- 退出码>1时等待（false返回1，所以不会等待）
     end
   }, callback)

5. 按 e 键：执行 echo，无回调
   lc.interactive({'echo', 'No callback'})

6. 按 E 键：执行 ls，总是等待，无回调
   lc.interactive({'ls'}, {wait_confirm = true})
]]
  cb(help)
end

return M
