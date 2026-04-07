# Preset Lua Scripts

本目录包含 lazycmd 的预设 Lua 脚本，这些脚本在应用启动时加载，为插件提供基础工具函数和 API 封装。

## 目录结构

```
preset/lua/
├── api.lua           # 页面管理 API 封装
├── base64.lua        # Base64 编解码
├── cache.lua         # 缓存 API 封装
├── clipboard.lua     # 系统剪贴板访问
├── component.lua     # UI 组件（对话框、通知）
├── copy_from_neovim.lua # 从 Neovim 复用的表工具函数
├── fs.lua            # 文件系统 API 封装
├── html.lua          # HTML 解析 API 封装
├── http.lua          # HTTP API 封装
├── http_server.lua   # 本地 HTTP 服务封装
├── init.lua          # 初始化脚本（默认配置和键盘映射）
├── inspect.lua       # 调试工具（表结构可视化）
├── interactive.lua   # 交互式命令封装
├── json.lua          # JSON 编解码
├── keymap.lua        # 键盘映射 API 封装
├── promise.lua       # 内置 Promise 实现与全局变量
├── secrets.lua       # secrets API 封装
├── socket.lua        # 长连接 socket 封装
├── string.lua        # 字符串扩展方法
├── style.lua         # 样式 API 封装
├── system.lua        # 系统命令 API 封装
├── time.lua          # 时间 API 封装
├── url.lua           # URL 编解码
├── util.lua          # 工具函数
├── yaml.lua          # YAML 编解码
└── global.d.lua      # 类型声明文件
```

## 概述

preset/lua 目录中的脚本是 Rust 后端 API 的 Lua 封装层。它们：

1. **提供更友好的 Lua API** - 封装底层 Rust 实现，添加类型注解和文档
2. **增加功能性** - 如 `json.lua`、`inspect.lua` 提供完整的 JSON 处理和调试能力
3. **统一接口** - 为不同调用方式提供统一的封装（如 `interactive.lua`、`system.lua`）

底层 Rust API 实现请参考 [src/plugin/README.md](../src/plugin/README.md)。

## 模块说明

### api.lua - 页面管理

封装页面和导航相关的 API。页面 entry 常用字段包括：

- `key: string` - 唯一标识
- `display?: string|Span|Line` - 列表区显示内容
- `bottom_line?: string|Span|Line` - 当前 entry 被 hover 时显示在底部左侧的一行
- `keymap?: table` - entry 局部快捷键
- `preview?: function` - entry 局部预览回调

```lua
lc.api.set_entries(path, entries)    -- path=nil 为当前页面，entries=nil 清空 Rust 侧页面
lc.api.get_entries(path)             -- path=nil 为当前页面
lc.api.get_hovered()                 -- 获取当前悬停项
lc.api.set_hovered(path)             -- 按完整路径设置当前悬停项
lc.api.set_preview(path, widget)     -- path=nil 为当前悬停项，widget=nil 清空 Rust 侧预览缓存
lc.api.go_to(path)                   -- 导航到路径
lc.api.get_current_path()            -- 获取当前路径
lc.api.get_hovered_path()            -- 获取悬停项完整路径
lc.api.argv()                        -- 获取命令行参数
lc.api.get_filter()                  -- 获取当前过滤字符串
lc.api.set_filter()                  -- 设置当前过滤字符串
lc.hook.pre_reload(cb)               -- 添加重载前钩子
```

### cache.lua - 缓存系统

持久化缓存的 Lua 封装：

```lua
lc.cache.get(namespace, key)           -- 获取缓存
lc.cache.set(namespace, key, value, opts)  -- 设置缓存（支持 TTL）
lc.cache.delete(namespace, key)        -- 删除缓存
lc.cache.clear(namespace)              -- 清空指定 namespace 的缓存
```

### clipboard.lua - 系统剪贴板

系统剪贴板访问封装：

```lua
lc.clipboard.get()         -- 获取剪贴板内容
lc.clipboard.set(text)     -- 设置剪贴板内容
```

### base64.lua - Base64 编解码

Base64 相关封装：

```lua
lc.base64.encode("hello")                  -- 编码
lc.base64.decode("aGVsbG8=")               -- 解码为 Lua 字符串
lc.fs.write_file_sync("/tmp/a.bin", lc.base64.decode("...")) -- 解码后写入文件
```

### component.lua - UI 组件

对话框和通知组件：

```lua
lc.select(opts, on_selection)  -- 选择对话框
lc.confirm(opts)              -- 确认对话框
lc.notify(message)            -- 通知消息 (支持 string、Span、Line 或 Text 类型)
lc.log(level, format, ...)   -- 写入日志
```

### fs.lua - 文件系统

文件系统操作封装：

```lua
lc.fs.read_dir_sync(path)     -- 读取目录（每项包含 name / is_dir / size）
lc.fs.read_file(path, callback)  -- 异步读取文件
lc.fs.read_file(path, { max_chars = 20000 }, callback)  -- 限制最多读取字符数
lc.fs.read_file_sync(path)    -- 读取文件
lc.fs.write_file_sync(path, content)  -- 写入文件（支持二进制内容）
lc.fs.stat(path)              -- 获取文件状态（包含 size 字段）
lc.fs.mkdir(path)             -- 创建目录
```

### secrets.lua - Secrets 存储

敏感字符串持久化封装：

```lua
lc.secrets.get(namespace, key)    -- 获取 secret
lc.secrets.set(namespace, key, value) -- 保存 secret
lc.secrets.delete(namespace, key) -- 删除 secret
```

### http.lua - HTTP 客户端

异步 HTTP 请求封装：

```lua
lc.http.get(url, callback)
lc.http.post(url, body, callback)
lc.http.put(url, body, callback)
lc.http.delete(url, callback)
lc.http.patch(url, body, callback)
lc.http.request(opts, callback)
```

### http_server.lua - 本地 HTTP 服务

本地 HTTP 服务封装，适合为外部进程或插件生成稳定 localhost URL：

```lua
lc.http_server.register_resolver('song', function(req, respond)
  respond {
    status = 307,
    headers = {
      Location = 'https://example.com/signed-url',
    },
  }
end)

local url = lc.http_server.url('song', { id = 123 })
local info = lc.http_server.info()
lc.http_server.unregister_resolver('song')
```

### html.lua - HTML 解析

HTML 文档/片段解析与 CSS selector 查询封装：

```lua
local doc = lc.html.parse(response.body)
local repos = doc:select("article.Box-row")
local first = repos[1]

if first then
  local link = first:first("h2 a")
  local href = link and link:attr("href")
  local text = link and link:text()
end
```

### url.lua - URL 编解码

URL 百分号编码封装：

```lua
lc.url.encode("hello world")  -- "hello%20world"
lc.url.decode("hello%20world") -- "hello world"
```

### init.lua - 初始化脚本

应用启动时执行的默认初始化：

- 根据 `plugins` 配置把本地 `dir` 和远程安装目录加入 `package.path`
- 根路径 `/` 固定展示所有已配置插件
- 进入 `/plugin_name/...` 时懒加载该插件并执行其 `config/setup`
- 根据 `cfg.keymap` 注册默认主模式键盘映射
- 加载用户配置（通过 `require 'init'`）
- 实现 `lc._list()` 和 `lc._preview()` 入口函数

默认配置中的 `keymap` 字段：

```lua
{
  up = '<up>',
  down = '<down>',
  top = 'gg',
  bottom = 'G',
  preview_up = '<pageup>',
  preview_down = '<pagedown>',
  reload = '<C-r>',
  history_back = '<C-o>',
  quit = 'q',
  command_prompt = ':',
  force_quit = '<C-q>',
  filter = '/',
  clear_filter = '<esc>',
  back = '<left>',
  open = '<right>',
  enter = '<enter>',
}
```

默认键盘映射：

| 按键 | 命令 |
|------|------|
| `↑` | 向上滚动 |
| `↓` | 向下滚动 |
| `gg` | 跳到开头 |
| `G` | 跳到结尾 |
| `<PageUp>` | 预览向上滚动 |
| `<PageDown>` | 预览向下滚动 |
| `Ctrl+r` | 刷新 |
| `Ctrl+o` | 跳回上一个访问页面 |
| `:` | 打开命令输入框 |
| `q` | 退出 |
| `/` | 进入过滤模式 |
| `?` | 打开快捷键帮助 |
| `Esc` | 清除过滤 |
| `←` | 返回上级 |
| `→` / `Enter` | 进入目录 |

### inspect.lua - 调试工具

将任意 Lua 值转换为可读字符串（基于 `inspect.lua` 库）：

```lua
lc.inspect(value, options)
-- options: depth, newline, indent, process
```

### interactive.lua - 交互式命令

封装交互式命令执行，支持多种调用格式：

```lua
lc.interactive({"cmd", "arg1"})
lc.interactive({"cmd"}, callback)
lc.interactive({"cmd"}, {wait_confirm = true})
lc.interactive({"cmd"}, {wait_confirm = function(code) return code ~= 0 end})
lc.interactive({"cmd"}, {wait_confirm = true}, callback)
```

### json.lua - JSON 处理

完整的 JSON 编解码库（基于 rxi 的 json.lua）：

```lua
lc.json.encode(value)   -- Lua 值转 JSON 字符串
lc.json.decode(str)     -- JSON 字符串转 Lua 值
```

### promise.lua - Promise

启动时会直接加载内置 `Promise`，推荐直接使用全局变量：

```lua
Promise                -- 全局变量
```

兼容旧代码时，`require('promise')` 也会返回同一个 Promise 表；正常情况下不需要 `require`，也不需要再把 `promise` 作为插件加入 `lc.config.plugins`。

### keymap.lua - 键盘映射

设置键盘快捷键：

```lua
lc.keymap.set(mode, key, callback[, opt])
-- mode: "main" 或 "input"
-- key: 键序列（如 "j", "<C-d>", "<down>"）
-- callback: 命令字符串或回调函数
-- opt.desc: 可选描述，用于帮助面板
-- opt.once: 可选布尔值，触发一次后自动删除
```

```lua
lc.keymap.set('main', '?', function() end, { desc = 'help' })
lc.keymap.set('main', 'p', function() end, { once = true, desc = 'paste once' })
```

`lc.config` 支持通过 `keymap` 字段覆盖内置主模式快捷键：

```lua
lc.config {
  keymap = {
    enter = '<enter>',
    filter = '/',
    quit = 'q',
  },
}
```

输入框默认键位也通过 `lc.config.keymap` 配置，而不是写死在 Rust 中，例如：

```lua
lc.config {
  keymap = {
    input_submit = '<enter>',
    input_cancel = '<esc>',
  },
}
```

其中 `Backspace`、`Left`、`Right`、`Ctrl-G` 是 Rust 内置输入键位，不通过 `lc.config.keymap` 配置。`Ctrl-G` 会调用外部编辑器编辑当前输入内容，优先使用 `$VISUAL`，其次 `$EDITOR`，否则回退到 `vi`。

支持的字段有：`up`、`down`、`top`、`bottom`、`preview_up`、`preview_down`、`reload`、`history_back`、`quit`、`force_quit`、`command_prompt`、`filter`、`clear_filter`、`back`、`open`、`enter`、`input_submit`、`input_cancel`、`input_clear_before_cursor`、`input_cursor_to_start`、`input_cursor_to_end`。每次调用 `lc.config` 都会根据当前 `keymap` 重新调用一遍 `lc.keymap.set`。

通过 `:` 打开的命令输入框可以执行内部命令，例如：

```lua
cd /github/search
cd ../repo/lazygit
reload
history_back
```

其中 `cd` 支持绝对路径（以 `/` 开头）、相对路径，以及 `.` / `..`。

页面 entry 还可以定义 `keymap` 字段：

```lua
{
  key = "item",
  keymap = {
    ["x"] = { callback = function() print("entry local action") end, desc = "run action" },
  },
}
```

优先级顺序是：`entry.keymap` > 一次性 keymap（`opt.once = true`）> 普通全局 keymap。
一次性 keymap 在完整触发一次后会自动删除；如果之前有相同按键的普通全局 keymap，删除后会恢复到普通全局 keymap。
`entry.keymap` 的值也可以写成 `{ callback = fn, desc = "..." }`，供帮助面板展示描述。

可以通过 `lc.api.get_available_keymaps()` 获取当前上下文下可用的 entry/once/global 快捷键列表。

### plugin setup helper

可以主动触发某个已声明插件的 `setup/config`：

```lua
lc.plugin.load('mpv')
```

这会按 `lc.config.plugins` 里对应插件的配置执行一次 setup，适合一个插件依赖另一个插件的初始化结果时使用。

### hook helpers

```lua
lc.hook.pre_quit(function() ... end)
lc.hook.post_page_enter(function(ctx) print(vim.inspect(ctx.path)) end)
```

页面 entry 还可以定义 `preview` 字段：

```lua
{
  key = "item",
  preview = function(self, cb)
    cb(lc.style.text {
      lc.style.line { "Preview for " .. self.key },
    })
  end,
}
```

当光标停在该 entry 上且 `entry.preview` 存在时，会优先于插件级 `preview(entry, cb)`。如果回调执行时 hovered entry 已经变化，这次预览更新会被自动忽略。

`preview` 支持：
- `string` / `Span` / `Line` / `Text`
- `Image`
- 以上类型组成的数组，会在预览区按顺序渲染，适合图文混排

`Image` 会优先使用终端原生图片协议（当前支持 Kitty / iTerm Inline），不支持时退回 truecolor 块字符。

`entry.preview` 既可以异步调用 `cb(preview)`，也可以直接 `return preview` 返回同步结果：

```lua
{
  key = "item",
  preview = function(self)
    return lc.style.text {
      lc.style.line { "Immediate preview for " .. self.key },
    }
  end,
}
```

### string.lua - 字符串扩展

为字符串添加方法：

```lua
"text".fg("blue")       -- 设置前景色
"text":bold()           -- 加粗
"text":italic()         -- 斜体
"text":underline()      -- 下划线
"text":ansi()           -- 解析 ANSI 转义序列
"a,b,c":split(",")      -- 分割字符串
"  hello  ":trim()       -- 去除首尾空白
"你好世界":utf8_sub(1, 3)  -- UTF-8 字符截取
```

`utf8_sub()` 在 Lua 5.3+/5.4 下使用内置 `utf8` 库，在 LuaJIT 下会自动退回到内置的兼容实现。

支持的颜色：`black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`

### style.lua - 样式系统

创建 TUI 组件：

```lua
lc.style.span(s)              -- 创建 Span
lc.style.line({s1, s2, ...}) -- 创建 Line
lc.style.text({l1, l2, ...}) -- 创建 Text
lc.style.image(path, opts)   -- 创建 Image
lc.style.highlight(code, lang)  -- 语法高亮
lc.style.align_columns(lines)    -- 列对齐

lc.style.span("x"):bold()        -- Span 加粗
lc.style.line({"x"}):italic()    -- Line 斜体
lc.style.span("x"):underline()   -- Span 下划线

图片预览示例：

```lua
return {
  lc.style.line { "Cover" },
  lc.style.image("/tmp/cover.png", { max_height = 20 }),
  "",
  lc.style.text { "Some description" },
}
```
```

### system.lua - 系统命令

执行外部命令：

```lua
lc.system.exec({cmd = {"ls", "-la"}, callback = function(out) end})
lc.system.exec({"ls", "-la"}, function(out) end)
local pid = lc.system.spawn({"mpv", "--idle=yes"})
lc.system.kill(pid) -- 默认发送 SIGTERM
lc.system.executable("rustc")  -- 检查命令是否存在
lc.system.open("file.txt")       -- 用默认应用打开文件
```

### socket.lua - 长连接 Socket

复用 Unix socket 连接：

```lua
local sock = lc.socket.connect("unix:/tmp/test.sock")
sock:on_line(function(line) print(line) end)
sock:write("hello")
sock:close()
```

### time.lua - 时间处理

时间解析和格式化：

```lua
lc.time.now()                      -- 当前时间戳
lc.time.parse("2023-12-25T15:30:45Z")  -- 解析时间字符串
lc.time.format(1704067200)         -- 格式化时间戳
lc.time.format(1704067200, "compact")  -- 紧凑格式
lc.time.format(1704067200, "relative") -- 相对时间格式
```

### util.lua - 工具函数

通用工具函数：

```lua
lc.osc52_copy(text)        -- 复制到剪贴板
```

### copy_from_neovim.lua - 表工具函数

从 Neovim 代码中整理出的通用表操作函数：

```lua
lc.tbl_isempty(t)                     -- 判断表是否为空
lc.islist(t)                          -- 判断是否为连续数组
lc.tbl_extend('force', a, b, ...)     -- 浅合并多个表
lc.tbl_deep_extend('force', a, b, ...) -- 深合并多个表
lc.deep_equal(a, b)                   -- 深度比较
lc.tbl_map(func, t)                   -- 表值映射
lc.tbl_filter(func, t)                -- 过滤列表
lc.list_extend(dst, src)              -- 列表追加
```

### plugin_manager.lua - 插件管理核心

插件管理的核心逻辑，提供 GitHub 插件的安装、更新、锁文件管理等功能。挂载在 `lc._pm`：

```lua
-- 解析插件声明为标准化结构
-- 支持三种输入格式：字符串、表（单字符串）、表（完整配置）
local spec = lc._pm.parse_plugin_spec('owner/plugin.lazycmd')
-- spec.name = 'plugin'
-- spec.url = 'https://github.com/owner/plugin.lazycmd.git'
-- spec.install_path = '~/.local/share/lazycmd/plugins/plugin.lazycmd'
-- spec.config = auto-generated function() require('plugin').setup() end

local local_spec = lc._pm.parse_plugin_spec({ dir = 'plugins/my-plugin.lazycmd' })
-- local_spec.name = 'my-plugin'
-- local_spec.dir = '<config-base>/plugins/my-plugin.lazycmd'
-- local_spec.is_remote = false

-- 展开插件列表（去重，保留首次出现顺序）
local flat = lc._pm.flatten_plugins(plugins)
-- flat[1].name = 'plugin-a'
-- flat[2].name = 'plugin-b'

-- 并行安装缺失的插件
lc._pm.install_missing(plugins, callback)

-- 并行更新所有插件（遵循约束）
lc._pm.update_all(plugins, callback)

-- 根据锁文件恢复插件
lc._pm.restore_all(plugins, callback)

-- 安装单个插件
lc._pm.install(spec, callback)

-- 更新单个插件
lc._pm.update(spec, callback)

-- 检查插件是否有更新
lc._pm.check_update(spec, callback)

-- 读取/写入锁文件
local lock = lc._pm.read_lock()
lc._pm.write_lock(lock)
```

### manager.lua - 插件管理器 UI

内置插件管理界面。启动 lazycmd 后自动加载。提供以下函数：

```lua
local manager = lc._manager
manager.setup(plugins)  -- 初始化并设置键盘映射
manager.list(path, cb)  -- 列出所有插件
manager.preview(entry, cb)  -- 显示插件详情和更新状态
```

### global.d.lua - 类型声明

Lua 语言服务器类型声明文件，为 IDE 提供类型提示。

## 加载顺序

预设文件按以下顺序加载（参见 `src/plugin/lua.rs`）：

1. `system.lua`
2. `copy_from_neovim.lua`
3. `socket.lua`
4. `component.lua`
5. `api.lua`
6. `style.lua`
7. `interactive.lua`
8. `string.lua`
9. `inspect.lua`
10. `json.lua`
11. `time.lua`
12. `keymap.lua`
13. `http.lua`
14. `cache.lua`
15. `fs.lua`
16. `util.lua`
17. `base64.lua`
18. `url.lua`
19. `clipboard.lua`
20. `yaml.lua`
21. `plugin_manager.lua` ← 插件管理核心逻辑（提供 `lc._pm`）
22. `manager.lua` ← 插件管理器 UI（提供 `lc._manager`）
23. `init.lua` ← 最后加载，执行初始化逻辑

## 使用示例

```lua
-- 自定义插件示例

-- 使用 JSON 处理
local data = lc.json.encode({name = "test", value = 42})
local decoded = lc.json.decode(data)

-- 使用 HTTP 请求
lc.http.get("https://api.example.com/data", function(resp)
    if resp.success then
        local data = lc.json.decode(resp.body)
        -- 处理数据
    end
end)

-- 创建带样式的文本
local header = lc.style.line({
    lc.style.span("文件列表").fg("green"),
    lc.style.span(" (" .. count .. ")").fg("gray")
})
lc.api.set_preview(nil, lc.style.text({header, content}))

-- 语法高亮
local code = [[
function hello()
    print("Hello, World!")
end
]]
local highlighted = lc.style.highlight(code, "lua")
lc.api.set_preview(nil, highlighted)

-- 异步执行命令
lc.system.exec({"ls", "-la"}, function(out)
    print("Exit code:", out.code)
    print("Output:", out.stdout)
end)

-- 使用缓存
lc.cache.set("demo", "api_result", {data = "something"}, {ttl = 300})
local cached = lc.cache.get("demo", "api_result")

-- 交互式确认
lc.confirm({
    title = "确认删除",
    prompt = "确定要删除这个文件吗？",
    on_confirm = function()
        -- 执行删除
    end,
    on_cancel = function()
        -- 取消操作
    end
})

-- 选择对话框
lc.select({
    prompt = "选择操作",
    options = {
        {value = "open", display = "📂 打开"},
        {value = "edit", display = "✏️ 编辑"},
        {value = "delete", display = "🗑️ 删除"}
    }
}, function(choice)
    if choice then
        -- 处理选择
    end
end)

-- 格式化时间
local timestamp = lc.time.now()
local formatted = lc.time.format(timestamp, "compact")  -- "14:30" 或 "03/15" 或 "2024/03"
local relative = lc.time.format(timestamp - 3600, "relative")  -- "1 hour ago"
```
