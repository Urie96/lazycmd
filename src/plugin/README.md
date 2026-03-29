# Plugin System

lazycmd 的插件系统基于 Lua 运行时，允许用户通过 Lua 脚本扩展应用功能。

## 目录结构

```
src/plugin/
├── mod.rs      # 模块声明
├── lua.rs      # Lua 初始化和预设加载
├── scope.rs    # 作用域管理函数
└── lc/         # Lua API 子模块
    ├── api.rs          # 页面管理 API
    ├── cache.rs        # 缓存系统
    ├── fs.rs           # 文件系统操作
    ├── highlighter.rs  # 语法高亮
    ├── http.rs         # HTTP 客户端
    ├── keymap.rs       # 键盘映射
    ├── path.rs         # 路径操作
    ├── style.rs        # UI 样式
    ├── system.rs       # 系统命令执行
    └── time.rs         # 时间解析和格式化
```

## 核心组件

### lua.rs - Lua 初始化

负责初始化 Lua 环境和加载预设文件：

- `init_lua()` - 初始化 Lua 环境并设置配置基准目录
- 预设加载顺序包含 `util.lua`、`copy_from_neovim.lua`、`init.lua` 等基础模块
- `package.path` 由 `preset/lua/config.lua` 根据 `plugins` 配置动态追加

**插件路径搜索顺序**（`package.path`）：

| 来源 | 路径 | 说明 |
|------|------|------|
| 本地（显式 `dir`） | 配置中的相对/绝对路径 | 通过 `{ dir = '...' }` 注入到 `package.path` |
| **远程** | `~/.local/share/lazycmd/plugins/` | **从 GitHub 下载的插件** |
| 预设 | `preset/lua/` | 内置预设脚本（嵌入二进制） |

**远程插件目录结构**：
```
~/.local/share/lazycmd/plugins/
└── owner-plugin.lazycmd/
    └── owner-plugin/
        └── init.lua       # 插件入口
~/.config/lazycmd/plugins.lock        # 插件版本锁文件
```

加载预设文件（debug 模式从文件读取，release 模式从嵌入的二进制读取）：

### scope.rs - 作用域管理

提供 Lua 与 Rust 状态交互的桥梁：

```rust
scope(lua, state, sender, || {
    // Lua 代码可以访问 state 和 sender
})?
```

关键函数：
- `scope()` - 在作用域中执行 Lua 代码
- `borrow_scope_state()` - 不可变访问状态
- `mut_scope_state()` - 可变访问状态
- `send_render_event()` - 触发渲染更新
- `send_command()` - 发送内部命令

## LC API

全局表 `lc` 提供以下子系统：

### lc.api - 页面管理

| 函数 | 说明 |
|------|------|
| `page_set_entries(entries)` | 设置页面条目列表 |
| `page_get_entries()` | 获取当前页面完整条目列表（过滤前） |
| `page_get_hovered()` | 获取当前悬停条目 |
| `page_set_preview(preview)` | 设置预览内容 |
| `go_to(path)` | 导航到指定路径 |
| `get_current_path()` | 获取当前路径 |
| `get_hovered_path()` | 获取悬停项路径 |
| `argv()` | 获取命令行参数 |
| `get_filter()` | 获取当前过滤条件 |
| `get_available_keymaps()` | 获取当前上下文可用快捷键 |
| `enter_filter_mode()` | 进入过滤模式 |
| `exit_filter_mode()` | 退出过滤模式 |
| `accept_filter()` | 应用过滤 |
| `append_hook_pre_reload(cb)` | 添加重载前钩子 |
| `append_hook_pre_quit(cb)` | 添加退出前钩子（Lua 侧封装为 `lc.hook.pre_quit`） |

### lc.cache - 缓存系统

基于 JSON 文件的持久化缓存：

| 函数 | 说明 |
|------|------|
| `cache.get(key)` | 获取缓存值 |
| `cache.set(key, value, opts)` | 设置缓存值（支持 TTL） |
| `cache.delete(key)` | 删除缓存 |
| `cache.clear()` | 清空所有缓存 |

```lua
-- 使用示例
lc.cache.set("user_data", {name = "test"}, {ttl = 3600})  -- TTL 为秒
local data = lc.cache.get("user_data")
```

### lc.fs - 文件系统

文件系统操作：

| 函数 | 说明 |
|------|------|
| `fs.read_dir_sync(path)` | 读取目录（返回数组） |
| `fs.read_file(path, [opts], callback)` | 异步读取文件内容，可限制最大字符数 |
| `fs.read_file_sync(path)` | 读取文件内容 |
| `fs.write_file_sync(path, content)` | 写入文件 |
| `fs.stat(path)` | 获取文件状态 |
| `fs.mkdir(path)` | 创建目录 |

`fs.stat()` 返回的表包含：
- `exists` - 文件是否存在
- `is_file` - 是否为文件
- `is_dir` - 是否为目录
- `is_readable` - 是否可读
- `is_writable` - 是否可写
- `is_executable` - 是否可执行

### lc.http - HTTP 客户端

基于 reqwest 的异步 HTTP 客户端：

| 函数 | 说明 |
|------|------|
| `http.get(url, callback)` | GET 请求 |
| `http.post(url, body, callback)` | POST 请求 |
| `http.put(url, body, callback)` | PUT 请求 |
| `http.delete(url, callback)` | DELETE 请求 |
| `http.patch(url, body, callback)` | PATCH 请求 |
| `http.request(opts, callback)` | 通用请求 |

回调接收的响应对象：
```lua
function on_response(response)
    -- response.success  - 请求是否成功
    -- response.status   - HTTP 状态码
    -- response.body     - 响应体
    -- response.headers  - 响应头
    -- response.error    - 错误信息
end
```

### lc.keymap - 键盘映射

| 函数 | 说明 |
|------|------|
| `lc.keymap.set(mode, key, callback[, opt])` | 设置键盘映射 |

```lua
lc.keymap.set('main', 'q', function() lc.cmd('quit') end)
lc.keymap.set('main', 'j', 'scroll_by 1')
lc.keymap.set('input', '<C-k>', function() lc.notify('input keymap hit') end)
lc.keymap.set('input', '<enter>', 'input_submit')
lc.keymap.set('main', '<C-x>', function() ... end)
lc.keymap.set('main', '?', function() ... end, { desc = 'help' })
lc.keymap.set('main', 'p', function() paste() end, { once = true, desc = 'paste once' })
```

- `mode` 支持 `main` / `m` 和 `input` / `i`
- 输入框中 `Backspace`、`Left`、`Right`、`Ctrl-G` 为 Rust 内置键位；其中 `Ctrl-G` 会调用外部编辑器编辑当前输入内容，优先使用 `$VISUAL`，其次 `$EDITOR`，否则回退到 `vi`
- 其余默认动作通过 `preset/lua/config.lua` 用 `lc.keymap.set('input', ...)` 注册到内部命令，例如 `input_submit`、`input_cancel`

`lc.config` 还支持 `keymap` 字段来覆盖内置主模式键位，例如：

```lua
lc.config {
  keymap = {
    enter = '<enter>',
    filter = '/',
    quit = 'q',
  },
}
```

支持的键位名包括 `up`、`down`、`top`、`bottom`、`preview_up`、`preview_down`、`reload`、`quit`、`force_quit`、`filter`、`clear_filter`、`back`、`open`、`enter`，以及 `input_submit`、`input_cancel`、`input_clear_before_cursor`、`input_cursor_to_start`、`input_cursor_to_end`。每次调用 `lc.config` 都会按这些配置重新执行一遍 `lc.keymap.set`。

页面 entry 也可以定义局部 keymap：

```lua
{
  key = "container-1",
  keymap = {
    ["d"] = { callback = function() delete_container("container-1") end, desc = "delete" },
    ["gg"] = function() open_logs("container-1") end,
  },
}
```

- `entry.keymap` 会通过 Lua 表访问，支持由元表 `__index` 提供
- key 是按键序列字符串，value 可以是 Lua 函数，或 `{ callback = fn, desc = "..." }`
- 优先级为：`entry.keymap` > `opt.once = true` 的一次性 keymap > 普通全局 keymap
- `opt.once = true` 时，该全局 keymap 完整触发一次后会自动删除；如果存在相同按键的普通全局 keymap，删除后会恢复到普通全局 keymap
- `opt.desc` 可为全局 keymap 提供帮助面板中的描述文本

`lc.api` 额外提供当前上下文可用快捷键查询：

```lua
local items = lc.api.get_available_keymaps()
for _, item in ipairs(items) do
  print(item.key, item.desc, item.source)
end
```

页面 entry 也可以定义局部 preview：

```lua
{
  key = "song-1",
  preview = function(self, cb)
    cb(lc.style.text {
      lc.style.line { "Preview for ", self.key },
    })
  end,
}
```

- `entry.preview(cb)` 的优先级高于插件级 `plugin.preview(entry, cb)`
- `entry.preview` 可以异步调用 `cb(preview)`，也可以直接返回一个 preview widget 作为同步结果
- `entry.preview` 同样通过 Lua 表访问，支持由元表 `__index` 提供
- 当异步回调返回时，如果当前 hovered entry 已经变化，这次 preview 更新会被丢弃

### lc.path - 路径操作

| 函数 | 说明 |
|------|------|
| `lc.path.split(path)` | 分割路径为数组 |
| `lc.path.join(path_list)` | 合并路径数组 |

### lc.style - UI 样式

创建 TUI 组件的函数：

| 函数 | 说明 |
|------|------|
| `lc.style.span(s)` | 创建单个 Span |
| `lc.style.line(args)` | 创建 Line（Span 数组） |
| `lc.style.text(args)` | 创建 Text（Line 数组） |
| `lc.style.highlight(code, lang)` | 语法高亮代码 |
| `lc.style.ansi(s)` | 解析 ANSI 转义序列 |
| `lc.style.align_columns(lines)` | 对齐列 |

### lc.system - 系统命令

| 函数 | 说明 |
|------|------|
| `lc.system.executable(cmd)` | 检查命令是否可执行 |
| `lc.system.spawn(cmd)` | 启动后台命令并返回 pid |
| `lc.system.kill(pid[, signal])` | 向进程发送信号，默认 `SIGTERM` |
| `lc.system.open(path)` | 用默认应用打开文件 |
| `lc.system.exec(opts)` | 异步执行命令 |
| `lc.system.interactive(opts)` | 执行交互式命令 |

### lc.socket - 长连接 Socket

| 函数 | 说明 |
|------|------|
| `lc.socket.connect(addr)` | 连接 socket，返回可复用连接对象 |

连接对象方法：
- `sock:on_line(cb)` - 注册逐行回调
- `sock:write(message)` - 写入一条消息（自动补 `\n`）
- `sock:close()` - 关闭连接

### lc.time - 时间处理

| 函数 | 说明 |
|------|------|
| `lc.time.parse(str)` | 解析时间字符串为 Unix 时间戳 |
| `lc.time.now()` | 获取当前 Unix 时间戳 |
| `lc.time.format(ts, fmt)` | 格式化时间戳 |

支持的时间格式：
- ISO 8601: `2023-12-25T15:30:45Z`
- RFC 3339: `2023-12-25T15:30:45+08:00`
- RFC 2822: `Mon, 25 Dec 2023 15:30:45 +0800`
- 日期: `2023-12-25`
- 紧凑格式: `compact`（自动适配显示格式）

### lc.cache - 缓存

| 函数 | 说明 |
|------|------|
| `lc.cache.get(key)` | 获取缓存 |
| `lc.cache.set(key, value, opts)` | 设置缓存 |
| `lc.cache.delete(key)` | 删除缓存 |
| `lc.cache.clear()` | 清空缓存 |

### 其他函数

| 函数 | 说明 |
|------|------|
| `lc.defer_fn(fn, ms)` | 延迟执行函数 |
| `lc.cmd(cmd)` | 发送内部命令 |
| `lc.split(s, sep)` | 分割字符串 |
| `lc.log(level, msg, ...)` | 写入日志 |
| `lc.osc52_copy(text)` | 通过 OSC 52 复制到剪贴板 |
| `lc.tbl_extend(behavior, ...)` | 浅合并多个表 |
| `lc.tbl_deep_extend(behavior, ...)` | 深合并多个表 |
| `lc.deep_equal(a, b)` | 深度比较两个值 |
| `lc.tbl_map(func, t)` | 映射表值 |
| `lc.tbl_filter(func, t)` | 过滤列表 |
| `lc.list_extend(dst, src)` | 追加列表内容 |
| `lc.notify(msg)` | 显示通知 (支持 string、Span、Line 或 Text 类型) |
| `lc.confirm(opts)` | 显示确认对话框 |
| `lc.select(opts, callback)` | 显示选择对话框 |

## 内部命令

通过 `lc.cmd()` 发送：

| 命令 | 说明 |
|------|------|
| `quit` | 退出应用 |
| `scroll_by [n]` | 滚动列表 n 行 |
| `scroll_preview_by [n]` | 滚动预览 n 行 |
| `reload` | 刷新当前列表 |
| `enter_filter_mode` | 进入过滤模式 |
| `input_submit` | 提交输入框 |
| `input_cancel` | 取消输入框 |
| `input_clear_before_cursor` | 删除光标前所有文本 |
| `input_cursor_to_start` | 输入框光标移动到开头 |
| `input_cursor_to_end` | 输入框光标移动到结尾 |
| `exit_filter_mode` | 退出过滤模式 |
| `accept_filter` | 应用过滤 |
| `filter_clear` | 清除过滤 |

## 语法高亮

使用 syntect 库支持 180+ 种语言：

```lua
local code = [[
function hello() {
    print("Hello World");
}
]]
local highlighted = lc.style.highlight(code, "javascript")
lc.api.page_set_preview(highlighted)
```

## 使用示例

```lua
-- 自定义插件示例
local M = {}

function M.setup()
    -- 设置键盘映射
    lc.keymap.set('main', 'r', function()
        lc.cmd('reload')
    end)
    
    -- 异步获取数据
    lc.http.get("https://api.example.com/data", function(resp)
        if resp.success then
            local data = lc.json.decode(resp.body)
            -- 处理数据
        end
    end)
end

function M.list(path, cb)
    -- 列出目录内容
    lc.fs.read_dir_sync(path, function(entries, err)
        if err then
            cb({})
            return
        end
        -- 转换为 PageEntry 格式
        local result = {}
        for _, e in ipairs(entries) do
            table.insert(result, {
                key = e.name,
                display = e.is_dir and e.name .. "/" or e.name
            })
        end
        cb(result)
    end)
end

return M
```
