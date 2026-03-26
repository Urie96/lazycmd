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
├── fs.lua            # 文件系统 API 封装
├── http.lua          # HTTP API 封装
├── init.lua          # 初始化脚本（默认配置和键盘映射）
├── inspect.lua       # 调试工具（表结构可视化）
├── interactive.lua   # 交互式命令封装
├── json.lua          # JSON 编解码
├── keymap.lua        # 键盘映射 API 封装
├── socket.lua        # 长连接 socket 封装
├── string.lua        # 字符串扩展方法
├── style.lua         # 样式 API 封装
├── system.lua        # 系统命令 API 封装
├── time.lua          # 时间 API 封装
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

封装页面和导航相关的 API：

```lua
lc.api.page_set_entries(entries)  -- 设置页面条目
lc.api.page_get_hovered()         -- 获取当前悬停项
lc.api.page_set_preview(widget)    -- 设置预览内容
lc.api.go_to(path)                -- 导航到路径
lc.api.get_current_path()         -- 获取当前路径
lc.api.get_hovered_path()         -- 获取悬停项完整路径
lc.api.argv()                     -- 获取命令行参数
lc.api.get_filter()               -- 获取当前过滤字符串
lc.api.append_hook_pre_reload(cb) -- 添加重载前钩子
```

### cache.lua - 缓存系统

持久化缓存的 Lua 封装：

```lua
lc.cache.get(key)           -- 获取缓存
lc.cache.set(key, value, opts)  -- 设置缓存（支持 TTL）
lc.cache.delete(key)        -- 删除缓存
lc.cache.clear()            -- 清空缓存
```

### clipboard.lua - 系统剪贴板

系统剪贴板访问封装：

```lua
lc.clipboard.get()         -- 获取剪贴板内容
lc.clipboard.set(text)     -- 设置剪贴板内容
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
lc.fs.read_dir_sync(path)     -- 读取目录
lc.fs.read_file_sync(path)    -- 读取文件
lc.fs.write_file_sync(path, content)  -- 写入文件
lc.fs.stat(path)              -- 获取文件状态
lc.fs.mkdir(path)             -- 创建目录
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

### init.lua - 初始化脚本

应用启动时执行的默认初始化：

- 读取命令行参数确定默认插件
- 根据 `plugins` 配置把本地 `dir` 和远程安装目录加入 `package.path`
- 直接启动插件时，自动执行该插件在配置中的 `config/setup`
- 设置默认键盘映射
- 加载用户配置（通过 `require 'init'`）
- 实现 `lc._list()` 和 `lc._preview()` 入口函数

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
| `q` | 退出 |
| `/` | 进入过滤模式 |
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

### keymap.lua - 键盘映射

设置键盘快捷键：

```lua
lc.keymap.set(mode, key, callback)
-- mode: "main" 或 "input"
-- key: 键序列（如 "j", "<C-d>", "<down>"）
-- callback: 命令字符串或回调函数
```

页面 entry 还可以定义 `keymap` 字段：

```lua
{
  key = "item",
  keymap = {
    ["x"] = function() print("entry local action") end,
  },
}
```

当光标停在该 entry 上且 `entry.keymap` 对当前按键序列有前缀匹配时，会优先于全局 `lc.keymap.set`。

### string.lua - 字符串扩展

为字符串添加方法：

```lua
"text".fg("blue")       -- 设置前景色
"text":ansi()           -- 解析 ANSI 转义序列
"a,b,c":split(",")      -- 分割字符串
"  hello  ":trim()       -- 去除首尾空白
"你好世界":utf8_sub(1, 3)  -- UTF-8 字符截取
```

支持的颜色：`black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`

### style.lua - 样式系统

创建 TUI 组件：

```lua
lc.style.span(s)              -- 创建 Span
lc.style.line({s1, s2, ...}) -- 创建 Line
lc.style.text({l1, l2, ...}) -- 创建 Text
lc.style.highlight(code, lang)  -- 语法高亮
lc.style.align_columns(lines)    -- 列对齐
```

### system.lua - 系统命令

执行外部命令：

```lua
lc.system.exec({cmd = {"ls", "-la"}, callback = function(out) end})
lc.system.exec({"ls", "-la"}, function(out) end)
lc.system.spawn({"mpv", "--idle=yes"})
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
```

### util.lua - 工具函数

通用工具函数：

```lua
lc.tbl_map(func, t)       -- 表值映射
lc.tbl_extend(target, ...) -- 深度扩展表
lc.list_extend(dst, src)   -- 列表追加
lc.equals(o1, o2)          -- 深度比较
lc.osc52_copy(text)        -- 复制到剪贴板
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

-- 展开插件列表，包含依赖（去重、拓扑排序）
local flat = lc._pm.flatten_plugins(plugins)
-- flat[1].name = 'dep1'  -- 依赖优先
-- flat[2].name = 'main'  -- 主插件

-- 安装缺失的插件（包含依赖，按依赖顺序）
lc._pm.install_missing(plugins, callback)

-- 更新所有插件（遵循约束）
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

内置插件管理界面。当不带参数启动 lazycmd 时自动加载。提供以下函数：

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
2. `component.lua`
3. `api.lua`
4. `style.lua`
5. `interactive.lua`
6. `string.lua`
7. `inspect.lua`
8. `json.lua`
9. `time.lua`
10. `keymap.lua`
11. `http.lua`
12. `cache.lua`
13. `fs.lua`
14. `util.lua`
15. `base64.lua`
16. `clipboard.lua`
17. `yaml.lua`
18. `plugin_manager.lua` ← 插件管理核心逻辑（提供 `lc._pm`）
19. `manager.lua` ← 插件管理器 UI（提供 `lc._manager`）
20. `init.lua` ← 最后加载，执行初始化逻辑

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
lc.api.page_set_preview(lc.style.text({header, content}))

-- 语法高亮
local code = [[
function hello()
    print("Hello, World!")
end
]]
local highlighted = lc.style.highlight(code, "lua")
lc.api.page_set_preview(highlighted)

-- 异步执行命令
lc.system.exec({"ls", "-la"}, function(out)
    print("Exit code:", out.code)
    print("Output:", out.stdout)
end)

-- 使用缓存
lc.cache.set("api_result", {data = "something"}, {ttl = 300})
local cached = lc.cache.get("api_result")

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
```
