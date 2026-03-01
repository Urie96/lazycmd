# lazycmd AGENTS.md

本文档为在此 Rust 终端 UI 应用仓库中工作的 AI 实例提供项目特定信息。

## 插件开发工作流

lazycmd 的核心是其 Lua 插件系统。插件在启动时从 `preset/` 目录和用户配置目录加载。

### 调试预设 Lua 文件

在 debug 构建（`cargo build`）中，Lua 文件从磁盘读取：

- **预设文件**：直接从 `preset/` 目录读取
- **用户配置**：从 `examples/` 目录读取（`init.lua` 和 `plugins/`）

1. 运行 `cargo run -- process`
2. 编辑 `preset/` 中的文件（如 `preset/init.lua`, `preset/util.lua`）
3. 编辑 `examples/` 中的用户配置（如 `examples/init.lua`, `examples/plugins/`）
4. 重启应用以查看更改

在 release 构建（`cargo run --release`）中：

- **预设文件**：在编译时使用 `include_bytes!` 嵌入到二进制中
- **用户配置**：从 `~/.config/lazycmd/` 目录读取（`init.lua` 和 `plugins/`）

### 插件加载流程

1. `src/plugin/lua.rs::init_lua()` 在 `App::new()` 期间被调用
2. 设置 `package.path` 以包含用户配置目录
3. 调用 `lc::register()` 注册内置 LC API（包括 `lc.json`, `lc.inspect`）
4. 加载 `preset/util.lua` 设置工具函数（`tbl_map`, `tbl_extend`, `equals`, `trim`）
5. 加载 `preset/init.lua` 设置默认键盘映射和事件钩子
6. `preset/init.lua` 调用 `require 'init'` 加载用户配置文件
7. 用户配置通过 `lc.config()` 设置插件配置
8. 通过 `require()` 加载插件 - 参见 `examples/plugins/process.lazycmd/`

## 架构概览

### 核心应用流程

```
main.rs → App::new() → App::run() → 事件循环
```

- **运行时**：使用 `tokio::main(flavor = "current_thread")` - 单线程异步运行时
- **任务集**：主要逻辑在 `LocalSet` 中运行，用于本地任务生成

### 事件驱动架构

应用使用多流事件系统（`src/events.rs`）：

- **渲染流**：周期性渲染事件（默认 20 秒间隔，可在 `render_stream()` 中调整）
- **Crossterm 流**：终端输入事件（键盘、鼠标、调整大小）
- **文本流**：通过 MPSC 通道的内部应用事件

事件统一为 `enum Event` 并通过 `StreamMap` 处理。

### Lua 插件系统 - 关键架构

这是理解的最重要部分。插件系统在 Rust 和 Lua 之间建立桥梁：

#### 作用域模式（`src/plugin/scope.rs`）

Lua 代码在提供对 Rust 状态的可变引用的**作用域**中执行：

```rust
plugin::scope(&lua, &mut state, &sender, || {
    // 这里的 Lua 代码可以通过注册表访问 state 和 sender
    lua.global().get("lc")  // 访问 lc API
})?
```

- `state`（可变借用）：暴露为注册表值 `state`，可通过 `borrow_scope_state()` 和 `mut_scope_state()` 访问
- `sender`（EventSender）：暴露为注册表值 `sender`，允许 Lua 向 Rust 发送事件

#### LC API 结构（`src/plugin/lc/mod.rs`）

Lua 中的全局表 `lc` 提供以下子系统：

| Lua 模块        | 来源                     | 用途                                            |
| --------------- | ------------------------ | ----------------------------------------------- |
| `lc.api`        | `src/plugin/lc/api.rs`   | 页面管理（entries、预览、导航）、命令行参数访问 |
| `lc.fs`         | `src/plugin/lc/fs.rs`    | 同步文件系统操作                                |
| `lc.keymap`     | `src/plugin/lc/keymap.rs`| 注册键盘快捷键                                  |
| `lc.path`       | `src/plugin/lc/path.rs`  | 路径操作（split/join）                          |
| `lc.http`       | `src/plugin/lc/http.rs`  | HTTP 请求                                       |
| `lc.time`       | `src/plugin/lc/time.rs`  | 时间解析和格式化                                |
| `lc.json`       | `preset/json.lua`        | JSON 编解码                                     |
| `lc.inspect`    | `preset/inspect.lua`     | 调试输出                                        |
| `lc.defer_fn`   | `src/plugin/lc/mod.rs`   | 调度异步 Lua 回调                               |
| `lc.system`     | `src/plugin/lc/mod.rs`   | 异步执行外部命令                                |
| `lc.interactive`| `src/plugin/lc/mod.rs`   | 执行交互式命令（有终端访问权限）                |
| `lc.on_event`   | `src/plugin/lc/mod.rs`   | 注册事件钩子                                    |
| `lc.cmd`        | `src/plugin/lc/mod.rs`   | 向 Rust 发送内部命令                            |
| `lc.osc52_copy` | `src/plugin/lc/mod.rs`   | 通过 OSC 52 复制文本到剪贴板                    |
| `lc.notify`     | `src/plugin/lc/mod.rs`   | 在右下角显示通知消息（3 秒后自动消失）          |
| `lc.log`        | `src/plugin/lc/mod.rs`   | 写入日志文件                                    |
| `lc.split`      | `src/plugin/lc/mod.rs`   | 字符串分割                                      |
| `lc.tbl_map`    | `preset/util.lua`        | 对表值映射函数                                  |
| `lc.tbl_extend` | `preset/util.lua`        | 深度扩展表                                      |
| `lc.equals`     | `preset/util.lua`        | 深度比较值                                      |
| `lc.trim`       | `preset/util.lua`        | 去除字符串空白                                  |
| `lc.config`     | `preset/init.lua`        | 配置插件和设置                                  |

#### 事件钩子

插件可以挂钩到特定的应用事件：

```lua
lc.on_event('EnterPost', function() ... end)   -- 进入目录后
lc.on_event('HoverPost', function() ... end)  -- 改变选中项后
```

事件钩子通过 `Event::AddEventHook` 注册并在 `app.rs::run_event_hooks()` 中执行。

#### UI 字符串扩展

插件系统向 Lua 的字符串元表注入方法（`src/plugin/lc/ui.rs`）：

```lua
-- 设置文本颜色
"filename".fg("blue")  -- 返回一个 Span widget

-- 将 ANSI 转义序列解析为 TUI Text
ansi_string:ansi()  -- 返回一个 Text widget
```

### Widget 系统（`src/widgets/`）

自定义组件使用 Lua 集成扩展 ratatui：

- **Renderable trait**：任何可渲染到终端的基 trait
  - `StatefulParagraph`：带有滚动支持和滚动条的文本
  - `StatefulList`：带有滚动的列表 widget
- **FromLua trait**：允许从 Lua 值转换为 Rust widgets
  - 字符串 → `StatefulParagraph`
  - UserData（Text/Span）→ widgets
  - `PageEntry`：带有 `{key, display?, ...}` 的 Lua 表

### 键盘映射系统

键盘映射在 Lua 中配置并注册到 Rust：

```lua
-- 单字符键（直接写字符）
lc.keymap.set('main', 'd', function() ... end)
lc.keymap.set('main', 'x', function() ... end)
lc.keymap.set('main', 'q', function() ... end)

-- 角括号内的多字符键（功能键）
lc.keymap.set('main', '<C-x>', function() ... end)
lc.keymap.set('main', '<A-k>', function() ... end)
lc.keymap.set('main', '<up>', 'scroll_by -1')
lc.keymap.set('main', '<down>', 'scroll_by 1')
lc.keymap.set('main', '<enter>', function() ... end)

-- 多键序列（按顺序连续按键）
lc.keymap.set('main', 'dd', function() ... end)  -- 按 d d 触发
lc.keymap.set('main', '<C-x><C-c>', function() ... end)  -- 连续按键
lc.keymap.set('main', 'gg', 'scroll_to_top')
```

#### 支持的按键格式

| 格式                     | 示例                  | 说明                              |
| ------------------------ | --------------------- | --------------------------------- |
| 单字符                    | `d`, `x`, `j`         | 直接按键，作为单个字符                |
| 角括号格式（功能键）    | `<C-x>`, `<A-k>`, `<down>` | Vim 风格，用于功能键和带修饰符的键  |
| 多键序列                 | `dd`, `<C-x><C-c>`     | 连续按键，需要完全匹配才能触发        |
| 简写修饰符（角括号内） | `<C-x>`, `<A-k>`, `<S-tab>`  | 角括号内支持简写：C, A, S          |

#### 注意事项

- **多字符键名必须用角括号**：`<down>`, `<up>`, `<enter>`, `<tab>`, `<space>`, `<f5>` 等
- **单字符不需要角括号**：`d`, `x`, `j`, `k`, `q` 等
- **未包裹的 "down" 会被视为 4 个字符**：d, o, w, n
- **修饰符简写仅限角括号内**：`<C-x>` 有效，`ctrl-x` 也有效（向后兼容）

#### 支持的功能键

| 键名                     | 说明                  |
| ------------------------ | --------------------- |
| `<up>`, `<down>`, `<left>`, `<right>` | 方向键              |
| `<enter>`                 | 回车键                |
| `<esc>`                  | Esc 键                |
| `<tab>`, `<backtab>`      | Tab 和 Shift+Tab       |
| `<space>`                 | 空格键                |
| `<backspace>`             | 退格键                |
| `<delete>`                | Delete 键              |
| `<insert>`                | Insert 键              |
| `<home>`, `<end>`         | Home 和 End 键         |
| `<pageup>`, `<pagedown>`  | PageUp 和 PageDown 键    |
| `<f1>` ~ `<f12>`         | F1 ~ F12 功能键        |

#### 支持的修饰符

| 格式                     | 说明    | 示例                  |
| ------------------------ | ------- | --------------------- |
| `ctrl-` 或 `C-`  | Ctrl    | `<C-x>`, `<ctrl-a>`     |
| `alt-` 或 `A-`   | Alt     | `<A-k>`, `<alt-s>`      |
| `shift-` 或 `S-` | Shift   | `<S-enter>`, `<shift-tab>` |

Rust 通过**前缀匹配缓冲区**（`src/state.rs::tap_key()`）处理键盘事件：

1. 键盘事件累积在 `last_key_event_buffer` 中
2. 与注册的键盘映射匹配（按模式过滤）
3. 当只有一个键盘映射完全匹配时，执行其回调
4. 在无匹配或不匹配时清除缓冲区

### 状态管理

`src/state.rs` 中的 `State` 持有整个应用状态：

- `current_mode`：当前激活的模式（Main/Input）
- `current_path`：导航路径栈
- `current_page`：当前显示的 entries 列表
- `keymap_config`：注册的键盘绑定
- `last_key_event_buffer`：待处理的按键序列
- `current_preview`：预览面板内容（Renderable widget）

### 异步命令执行

外部命令作为 Tokio 任务生成（`src/plugin/lc/mod.rs::command_fn`）：

```lua
lc.system({'ls', '-la'}, function(output)
  -- output.code: number
  -- output.stdout: LuaString
  -- output.stderr: LuaString
end)
```

输出通过 `Event::LuaCallback` 发回，该事件重新进入 Lua 作用域。

### 应用布局

UI 布局在 `src/app.rs::AppWidget::render()` 中硬编码：

```
┌─────────────────────────────────┐
│ Header (height: 3)              │
├──────────┬──────────────────────┤
│          │                      │
│ List     │ Preview              │
│ (50%)    │ (remaining)          │
│          │                      │
├──────────┴──────────────────────┤
│ Notification (bottom-right)       │
└─────────────────────────────────┘
```

**通知区域**：位于预览区域右下角，用于显示临时通知消息，3 秒后自动消失。

## 插件示例：进程查看器

`examples/plugins/process.lazycmd/` 演示了一个完整的插件：

1. **列出**：使用 `lc.system({ 'ps', '-eo', 'pid,command' }, ...)` 获取进程列表
2. **预览**：使用 `lc.system({ 'pstree', '-p', pid }, ...)` 显示进程树
3. **集成**：从 `preset/init.lua` 事件钩子调用

### 插件配置文件结构

用户配置文件 `examples/init.lua`（debug 模式）或 `~/.config/lazycmd/init.lua`（release 模式）：

```lua
lc.config {
  plugins = {
    {
      'memos',
      config = function()
        require('memos').setup {
          token = 'your_token',
          base_url = 'https://your-memos-server.com',
        }
      end,
    },
    {
      'process',
      config = function() require('process').setup() end,
    },
  },
}
```

### 插件目录结构

```
examples/plugins/          (debug) 或 ~/.config/lazycmd/plugins/ (release)
├── memos.lazycmd/
│   └── memos/
│       └── init.lua
└── process.lazycmd/
    └── process/
        └── init.lua
```

**注意**：插件目录名使用 `*.lazycmd` 后缀，内部子目录与插件名相同，包含 `init.lua` 文件。

## LC API 参考

### lc.api 模块

页面管理和命令行参数访问函数：

| 函数                               | 参数             | 返回值               | 用途                                 |
| ---------------------------------- | ---------------- | -------------------- | ------------------------------------ |
| `lc.api.page_set_entries(entries)` | `Vec<PageEntry>` | `nil`                | 设置当前页面的条目列表               |
| `lc.api.page_get_hovered()`        | 无               | `PageEntry \| nil`   | 获取当前选中的条目                   |
| `lc.api.page_set_preview(widget)`  | `Renderable`     | `nil`                | 设置预览面板的内容                   |
| `lc.api.go_to(path)`               | `Vec<String>`    | `nil`                | 导航到指定路径                       |
| `lc.api.get_current_path()`        | 无               | `Vec<String>`        | 获取当前路径                         |
| `lc.api.get_hovered_path()`        | 无               | `Vec<String> \| nil` | 获取当前选中项的完整路径             |
| `lc.api.argv()`                    | 无               | `Vec<String>`        | 获取命令行参数（第一个元素为程序名） |
| `lc.api.append_hook_pre_reload(cb)` | `function`       | `nil`                | 注册 reload 命令前的钩子回调        |

**示例**：

```lua
-- 获取命令行参数
local args = lc.api.argv()
print("程序名称:", args[1])           -- 第一个元素是程序名
print("第一个参数:", args[2] or "无")  -- 第二个元素是第一个参数
print("所有参数:", lc.inspect(args))

-- 遍历所有参数
for i, arg in ipairs(lc.api.argv()) do
  print(i, arg)
end

-- 注册 pre_reload 钩子
lc.api.append_hook_pre_reload(function()
  lc.notify 'Reloading...'
  lc.log('info', 'Pre-reload hook triggered')
end)

-- 可以注册多个钩子，按注册顺序调用
lc.api.append_hook_pre_reload(function()
  -- 执行清理或验证逻辑
  lc.log('debug', 'Cleanup before reload')
end)
```

### lc.fs 模块

同步文件系统操作函数：

| 函数                         | 参数                     | 返回值                         | 用途                         |
| ---------------------------- | ------------------------ | ------------------------------ | ---------------------------- |
| `lc.fs.read_dir_sync(path)`  | `string`                 | `table[] \| string` (entries \| error) | 读取目录内容                 |
| `lc.fs.read_file_sync(path)` | `string`                 | `string \| string` (content \| error) | 读取文件内容                 |
| `lc.fs.write_file_sync(path, content)` | `string`, `string` | `boolean \| string` (success \| error) | 写入内容到文件               |

**示例**：

```lua
-- 读取文件内容
local content, err = lc.fs.read_file_sync('/tmp/memo.txt')
if err then
  lc.log('error', 'Failed to read file: {}', err)
else
  print('Content:', content)
end

-- 写入内容到文件
local success, err = lc.fs.write_file_sync('/tmp/memo.txt', 'Hello World')
if not success then
  lc.log('error', 'Failed to write file: {}', err)
end

-- 读取目录
local entries, err = lc.fs.read_dir_sync('/tmp')
if err then
  lc.log('error', 'Failed to read directory: {}', err)
else
  for _, entry in ipairs(entries) do
    print(entry.name)
  end
end
```

### lc.http 模块

HTTP 请求函数：

| 函数                       | 参数                           | 返回值                       | 用途              |
| -------------------------- | ------------------------------ | ---------------------------- | ----------------- |
| `lc.http.get(url, cb)`     | `string`, `function`           | `nil`                        | 发送 GET 请求    |
| `lc.http.post(url, body, cb)` | `string`, `string`, `function` | `nil`                    | 发送 POST 请求   |
| `lc.http.put(url, body, cb)` | `string`, `string`, `function` | `nil`                    | 发送 PUT 请求    |
| `lc.http.delete(url, cb)`  | `string`, `function`           | `nil`                        | 发送 DELETE 请求 |
| `lc.http.patch(url, body, cb)` | `string`, `string`, `function` | `nil`                   | 发送 PATCH 请求  |
| `lc.http.request(opts, cb)` | `RequestOptions`, `function` | `nil`                     | 发送自定义请求  |

**回调函数参数** (`HttpResponse`)：
- `success`: 请求是否成功
- `status`: HTTP 状态码
- `body`: 响应体
- `headers`: 响应头（table）
- `error`: 错误信息（如果有）

**示例**：

```lua
-- GET 请求
lc.http.get('https://api.example.com/data', function(res)
  if res.success then
    local data = lc.json.decode(res.body)
    lc.log('info', 'Received data: {}', lc.inspect(data))
  else
    lc.log('error', 'Request failed: {}', res.error)
  end
end)

-- POST 请求
lc.http.post('https://api.example.com/memos', lc.json.encode({
  content = 'Hello, world!',
  visibility = 'PRIVATE'
}), function(res)
  if res.success then
    lc.notify 'Memo created'
    lc.cmd 'reload'
  else
    lc.notify('Failed to create memo: ' .. (res.error or 'Unknown'))
  end
end)

-- 自定义请求
lc.http.request({
  url = 'https://api.example.com/memos',
  method = 'PATCH',
  headers = {
    ['Authorization'] = 'Bearer ' .. token,
    ['Content-Type'] = 'application/json',
  },
  body = lc.json.encode({content = 'Updated'}),
  timeout = 30000,
}, function(res)
  -- handle response
end)
```

### lc.json 模块

JSON 编解码函数：

| 函数                       | 参数     | 返回值   | 用途           |
| -------------------------- | -------- | -------- | -------------- |
| `lc.json.encode(value)`    | `any`    | `string` | 编码为 JSON 字符串 |
| `lc.json.decode(json_str)` | `string` | `any`    | 解码 JSON 为 Lua 值 |

**示例**：

```lua
-- 编码
local json_str = lc.json.encode({
  name = 'test',
  values = {1, 2, 3},
})
-- json_str = '{"name":"test","values":[1,2,3]}'

-- 解码
local data = lc.json.decode('{"name":"test","values":[1,2,3]}')
-- data = {name = "test", values = {1, 2, 3}}
```

### lc.osc52_copy 函数

通过 OSC 52 转义序列复制文本到系统剪贴板：

| 函数                        | 参数     | 返回值  | 用途                              |
| --------------------------- | -------- | ------- | --------------------------------- |
| `lc.osc52_copy(text)`       | `string` | `nil`   | 将文本复制到系统剪贴板            |

**说明**：

OSC 52 是一种终端转义序列，允许应用程序读写系统剪贴板。它通过标准输出向终端发送特殊的转义序列，终端将其转发到剪贴板管理器。

**注意事项**：

- 此功能要求终端模拟器支持 OSC 52 序列（大多数现代终端都支持，如 iTerm2、Kitty、WezTerm、Alacritty 等）
- 内容会先进行 base64 编码，然后通过转义序列发送
- 使用此功能可能需要在终端设置中启用剪贴板访问（某些默认禁用）

**示例**：

```lua
-- 复制备忘录内容到剪贴板
local entry = lc.api.get_hovered_entry()
if entry and entry.content then
  lc.osc52_copy(entry.content)
  lc.notify 'Copied to clipboard'
end

-- 复制自定义文本
lc.osc52_copy('Hello, world!')
```

### lc.notify 函数

在应用右下角显示临时通知消息：

| 函数             | 参数     | 返回值  | 用途              |
| ---------------- | -------- | ------- | ----------------- |
| `lc.notify(message)` | `string` | `nil`   | 显示通知消息（3 秒后自动消失） |

**说明**：

通知消息会显示在预览区域的右下角，带有边框。通知会自动在 3 秒后过期并从屏幕上消失。

**示例**：

```lua
-- 显示简单的通知
lc.notify('Copied to clipboard')

-- 显示操作结果通知
lc.notify('Memo updated successfully')

-- 显示错误信息
lc.notify('Failed to delete memo')
```

### lc.log 函数

写入日志到文件：

| 函数             | 参数                                   | 返回值  | 用途                    |
| ---------------- | -------------------------------------- | ------- | ----------------------- |
| `lc.log(level, format, ...)` | `string`, `string`, `...` | `nil`   | 写入日志到日志文件      |

**参数**：
- `level`: 日志级别（如 `"info"`, `"warn"`, `"error"`, `"debug"`）
- `format`: 格式化字符串，使用 `{}` 作为占位符
- `...`: 格式化参数

**日志位置**：`~/.local/state/lazycmd/lua.log`

**示例**：

```lua
lc.log('info', 'Loading memos list')
lc.log('debug', 'API call: {} {}', method, path)
lc.log('error', 'Failed to read file: {}', err)
```

### lc.split 函数

分割字符串：

| 函数             | 参数               | 返回值      | 用途           |
| ---------------- | ------------------ | ----------- | -------------- |
| `lc.split(s, sep)` | `string`, `string` | `string[]`  | 按分隔符分割字符串 |

**示例**：

```lua
local lines = lc.split("a,b,c", ",")  -- {"a", "b", "c"}
local lines = lc.split(output.stdout, "\n")
```

### lc.inspect 函数

将值转换为可读的字符串表示：

| 函数             | 参数    | 返回值    | 用途                   |
| ---------------- | ------- | --------- | ---------------------- |
| `lc.inspect(value)` | `any`  | `string`  | 返回值的调试输出字符串 |

**示例**：

```lua
print(lc.inspect(args))
local result = lc.inspect({a = 1, b = {c = 2}})
```

### lc.tbl_map 函数

对表值应用映射函数：

| 函数                    | 参数                     | 返回值   | 用途            |
| ----------------------- | ------------------------ | -------- | --------------- |
| `lc.tbl_map(func, t)`   | `function`, `table`      | `table`  | 返回映射后的新表 |

**示例**：

```lua
local doubled = lc.tbl_map(function(x) return x * 2 end, {1, 2, 3})
-- {2, 4, 6}
```

### lc.tbl_extend 函数

深度扩展目标表：

| 函数                        | 参数               | 返回值   | 用途              |
| --------------------------- | ------------------ | -------- | ----------------- |
| `lc.tbl_extend(target, ...)` | `table`, `table...` | `table`  | 返回扩展后的目标表 |

**示例**：

```lua
local config = {a = 1}
lc.tbl_extend(config, {b = 2}, {c = 3})
-- config = {a = 1, b = 2, c = 3}
```

### lc.equals 函数

深度比较两个值：

| 函数                          | 参数                    | 返回值    | 用途               |
| ----------------------------- | ----------------------- | --------- | ------------------ |
| `lc.equals(o1, o2, ignore_mt)` | `any`, `any`, `boolean?` | `boolean` | 值是否相等        |

**示例**：

```lua
if lc.equals(path, lc.api.get_current_path()) then
  lc.api.page_set_entries(entries)
end
```

### lc.trim 函数

去除字符串首尾空白：

| 函数             | 参数     | 返回值    | 用途           |
| ---------------- | -------- | --------- | -------------- |
| `lc.trim(s)`     | `string` | `string`  | 返回修剪后的字符串 |

**示例**：

```lua
local trimmed = lc.trim("  hello  ")  -- "hello"
```

### lc.config 函数

配置插件和设置：

| 函数                    | 参数     | 返回值  | 用途              |
| ----------------------- | -------- | ------- | ----------------- |
| `lc.config(opt)`        | `table`  | `nil`   | 配置插件          |

**配置选项**：
- `default_plugin`: 默认插件名称（来自命令行参数）
- `plugins`: 插件列表，每个插件包含：
  - `[1]`: 插件名称（如 `"memos"`, `"process"`）
  - `config`: 可选的配置函数

**示例**：

```lua
lc.config {
  plugins = {
    {
      'memos',
      config = function()
        require('memos').setup {
          token = 'your_token',
          base_url = 'https://your-server.com',
        }
      end,
    },
    {
      'process',
      config = function() require('process').setup() end,
    },
  },
}
```

### 内部命令

通过 `lc.cmd()` 可以发送内部命令到 Rust 端处理：

| 命令 | 参数 | 用途 |
|------|------|------|
| `quit` | 无 | 退出应用 |
| `scroll_by <num>` | 可选数字 | 列表滚动指定行数（默认 1） |
| `scroll_preview_by <num>` | 可选数字 | 预览面板滚动指定行数（默认 1） |
| `reload` | 无 | 刷新当前列表（重新调用插件的 `list()` 函数），执行前会调用所有 pre_reload 钩子 |
| `enter_filter_mode` | 无 | 进入过滤模式 |
| `exit_filter_mode` | 无 | 退出过滤模式 |
| `accept_filter` | 无 | 接受当前过滤 |
| `filter_clear` | 无 | 清除过滤条件 |

**示例**：

```lua
-- 使用命令字符串快捷方式
lc.keymap.set('main', 'j', 'scroll_by 1')
lc.keymap.set('main', 'k', 'scroll_by -1')

-- 使用 reload 命令
lc.keymap.set('main', 'ctrl-r', 'reload')

-- 使用过滤模式
lc.keymap.set('main', '/', 'enter_filter_mode')
lc.keymap.set('main', '<esc>', 'filter_clear')
```

## 日志

有两种日志：

### Rust 日志
使用 `tracing` crate 写入 `~/.local/state/lazycmd/lazycmd.log`：

```bash
# 实时查看日志
tail -f ~/.local/state/lazycmd/lazycmd.log
```

### Lua 日志
使用 `lc.log()` 函数写入 `~/.local/state/lazycmd/lua.log`：

```lua
lc.log('info', 'Loading memos list')
lc.log('debug', 'API call: {} {}', method, path)
lc.log('error', 'Failed to read file: {}', err)
```

查看 Lua 日志：
```bash
# 实时查看 Lua 日志
tail -f ~/.local/state/lazycmd/lua.log
```

## 添加新功能

### 添加新的 LC API 函数

**重要**：每次修改或添加新的 LC API 函数后，必须同步更新 `preset/types.lua` 文件，以保持类型定义与实现的一致性。

1. 在适当的 `src/plugin/lc/*.rs` 文件中添加函数
2. 在 `new_table()` 中注册它以添加到 lc 全局
3. 如果需要状态访问，使用 `plugin::borrow_scope_state()` 或 `plugin::mut_scope_state()`
4. 如果需要触发更新，调用 `plugin::send_render_event()`
5. 在 `preset/types.lua` 中添加对应的类型定义（ EmmyLua 注释格式）

### 添加新的内部命令

1. 在 `src/app.rs::handle_command()` 的 match 分支中添加新的命令名称
2. 实现命令逻辑，可能需要修改 state、触发事件钩子等
3. 如果命令会改变 UI，设置 `self.dirty = true` 以触发重新渲染
4. 在 `AGENTS.md` 中更新内部命令表格

**示例**：添加 `clear_preview` 命令清除预览面板

```rust
// 在 src/app.rs::handle_command() 中
"clear_preview" => {
    self.state.current_preview.take();
    self.dirty = true;
}
```

### 添加新的事件钩子

1. 在 `src/events.rs` 中向 `EventHook` 枚举添加变体
2. 如果需要，实现 `FromLua` trait
3. 在 `src/app.rs` 的适当位置调用 `self.run_event_hooks(...)`
4. 在代码注释中记录钩子用法

### 创建新插件

1. 在 `examples/plugins/myplugin.lazycmd/myplugin/` 下创建目录（debug 模式）
   或 `~/.config/lazycmd/plugins/myplugin.lazycmd/myplugin/`（release 模式）
2. 添加 `init.lua` 返回带有函数的表
3. 在 `examples/init.lua`（debug）或 `~/.config/lazycmd/init.lua`（release）中添加插件配置
4. 使用 `require 'myplugin'` 加载插件

**插件目录结构**：
```
examples/plugins/           (debug) 或 ~/.config/lazycmd/plugins/ (release)
└── myplugin.lazycmd/
    └── myplugin/
        └── init.lua
```

**插件示例**：
```lua
-- examples/plugins/myplugin.lazycmd/myplugin/init.lua
local M = {}

function M.setup()
  -- 设置键盘映射
  lc.keymap.set('main', 'x', function()
    lc.notify 'Plugin action'
  end)
end

function M.list(path, cb)
  -- 返回 entries 列表
  local entries = {
    {key = "1", display = "Item 1"},
    {key = "2", display = "Item 2"},
  }
  cb(entries)
end

function M.preview(entry, cb)
  -- 返回预览内容
  cb("Preview for: " .. entry.key)
end

return M
```

**用户配置**：
```lua
-- examples/init.lua
lc.config {
  plugins = {
    {'myplugin', config = function() require('myplugin').setup() end},
  },
}
```

## 重要实现细节

### Lua 生命周期

- `Lua` 实例在 `App` 的生命周期内存活
- Lua 函数（`LuaFunction`）存储在 Rust 结构体中（如 `Keymap`、`EventHook` 回调）
- 调用 Lua 回调时，始终用 `plugin::scope()` 包装以提供注册表值

### 错误处理

- 全程使用 `anyhow::Result<T>`
- 通过 `errors::install_hooks()` 安装更好的 Panic 钩子以获取详细回溯
- Panic 处理器中的终端清理通过 `term::restore()` 进行

### 模式系统

目前只有 `Main` 模式完全实现。`Input` 模式存在但未连接。未来的功能应扩展 `src/mode.rs` 中的 `Mode` 枚举，并在需要时添加相应的 `FromLua` 实现。

### 预设文件加载

**预设文件**（嵌入到二进制中）：

- `preset/util.lua` - 工具函数（tbl_map, tbl_extend, equals, trim）
- `preset/init.lua` - 默认初始化和键盘映射
- `preset/json.lua` - JSON 编解码
- `preset/inspect.lua` - 调试输出

`src/plugin/lua.rs` 中的宏 `preset!($name)` 处理 debug（文件读取）和 release（嵌入字节）两种构建。添加新预设文件时，确保将它们放置在 `preset/` 目录中。

**用户配置文件**（从磁盘读取）：

- debug 模式：`examples/init.lua`, `examples/plugins/`
- release 模式：`~/.config/lazycmd/init.lua`, `~/.config/lazycmd/plugins/`

用户配置文件通过 `preset/init.lua` 末尾的 `require 'init'` 加载。

### 键盘映射解析

键盘映射字符串在 `src/keymap.rs` 中解析，支持以下格式：

- 单字符键（无需角括号）：`"d"`, `"x"`, `"q"` 等
- 角括号内的功能键：`"<down>"`, `"<up>"`, `"<enter>"` 等
- 角括号内的带修饰符按键：`"<C-x>"`, `"<A-k>"`, `"<S-tab>"` 等
- 简写修饰符（仅角括号内）：`C-`, `A-`, `S-`
- 多键序列：`"dd"`, `"<C-x><C-c>"`, `"gg"` 等

**重要规则**：

1. 角括号外的任何内容（包括 `down`, `up`, `enter`）都会被当作单独字符解析
   - `"down"` → d, o, w, n（4 个字符）
   - `"<down>"` → Down（1 个功能键）

2. 角括号内的内容视为功能键名，支持修饰符简写
   - `"<C-x>"` → Ctrl+x
   - `"<A-k>"` → Alt+k
   - `"<down>"` → Down

3. 多键序列需要完全匹配才能触发
   - `"dd"` → 按两次 d
   - `"<C-x><C-c>"` → Ctrl+x 然后 Ctrl+c

键盘映射的单元测试位于 `src/keymap.rs` 的测试模块中。
