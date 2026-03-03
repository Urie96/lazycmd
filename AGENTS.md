# lazycmd AGENTS.md

本文档为在此 Rust 终端 UI 应用仓库中工作的 AI 实例提供项目特定信息。

## 常用命令

使用 `just` 进行开发：

```bash
cargo run -- <plugin_name>  # 运行指定插件（如 process, memos）
cargo build       # Debug 构建
cargo run --release           # Release 构建
cargo test        # 运行测试
cargo test <test_name>        # 运行单个测试
```

## 插件开发工作流

lazycmd 的核心是其 Lua 插件系统。

### 调试预设 Lua 文件

- **Debug 模式**：预设文件从 `preset/` 读取，用户配置从 `examples/` 读取
- **Release 模式**：预设文件嵌入二进制，用户配置从 `~/.config/lazycmd/` 读取

调试流程：
1. 修改 `preset/` 或 `examples/` 中的 Lua 文件
2. 重启应用查看更改

### 插件加载流程

1. `src/plugin/lua.rs::init_lua()` 初始化 Lua 环境
2. 加载 `preset/util.lua`、`preset/json.lua`、`preset/inspect.lua`、`preset/init.lua`
3. `preset/init.lua` 调用 `require 'init'` 加载用户配置
4. 用户配置通过 `lc.config()` 设置插件，通过 `require()` 加载插件

### 插件目录结构

```
examples/plugins/           (debug) 或 ~/.config/lazycmd/plugins/ (release)
└── myplugin.lazycmd/
    └── myplugin/
        └── init.lua
```

**注意**：插件目录名使用 `*.lazycmd` 后缀，内部子目录与插件名相同。

插件接口（init.lua 返回的表）：
- `setup()` - 初始化函数，设置键盘映射等
- `list(path, cb)` - 返回条目列表，cb 接收 entries 数组
- `preview(entry, cb)` - 返回预览内容，cb 接收 Renderable widget

## 架构概览

### 核心应用流程

```
main.rs → App::new() → App::run() → 事件循环
```

- **运行时**：`tokio::main(flavor = "current_thread")` - 单线程异步运行时
- **任务集**：主要逻辑在 `LocalSet` 中运行

### 事件驱动架构

多流事件系统（`src/events.rs`）：
- **渲染流**：周期性渲染事件
- **Crossterm 流**：终端输入事件（键盘、鼠标、调整大小）
- **文本流**：通过 MPSC 通道的内部应用事件

事件统一为 `enum Event` 并通过 `StreamMap` 处理。

### Lua 插件系统 - 关键架构

#### 作用域模式（`src/plugin/scope.rs`）

Lua 代码在提供对 Rust 状态的可变引用的**作用域**中执行：

```rust
plugin::scope(&lua, &mut state, &sender, || {
    // Lua 代码可通过注册表访问 state 和 sender
})?
```

- `state`：通过 `borrow_scope_state()` 和 `mut_scope_state()` 访问
- `sender`：允许 Lua 向 Rust 发送事件

#### LC API 结构

Lua 中的全局表 `lc` 提供以下子系统：

| Lua 模块        | 来源                     | 用途                          |
| --------------- | ------------------------ | ----------------------------- |
| `lc.api`        | `src/plugin/lc/api.rs`   | 页面管理、命令行参数访问      |
| `lc.fs`         | `src/plugin/lc/fs.rs`    | 同步文件系统操作              |
| `lc.keymap`     | `src/plugin/lc/keymap.rs`| 注册键盘快捷键                |
| `lc.path`       | `src/plugin/lc/path.rs`  | 路径操作                      |
| `lc.http`       | `src/plugin/lc/http.rs`  | HTTP 请求                     |
| `lc.time`       | `src/plugin/lc/time.rs`  | 时间解析和格式化              |
| `lc.json`       | `preset/json.lua`        | JSON 编解码                   |
| `lc.inspect`    | `preset/inspect.lua`     | 调试输出                      |
| `lc.defer_fn`   | `src/plugin/lc/mod.rs`   | 调度异步 Lua 回调             |
| `lc.system`     | `src/plugin/lc/mod.rs`   | 异步执行外部命令              |
| `lc.interactive`| `src/plugin/lc/mod.rs`   | 执行交互式命令                |
| `lc.on_event`   | `src/plugin/lc/mod.rs`   | 注册事件钩子                  |
| `lc.cmd`        | `src/plugin/lc/mod.rs`   | 向 Rust 发送内部命令          |
| `lc.osc52_copy` | `src/plugin/lc/mod.rs`   | 通过 OSC 52 复制文本到剪贴板  |
| `lc.notify`     | `src/plugin/lc/mod.rs`   | 显示通知消息                  |
| `lc.confirm`    | `src/plugin/lc/mod.rs`   | 显示确认对话框                |
| `lc.log`        | `src/plugin/lc/mod.rs`   | 写入日志文件                  |
| `lc.split`      | `src/plugin/lc/mod.rs`   | 字符串分割                    |
| `lc.tbl_map`    | `preset/util.lua`        | 对表值映射函数                |
| `lc.tbl_extend` | `preset/util.lua`        | 深度扩展表                    |
| `lc.equals`     | `preset/util.lua`        | 深度比较值                    |
| `lc.trim`       | `preset/util.lua`        | 去除字符串空白                |
| `lc.config`     | `preset/init.lua`        | 配置插件和设置                |

#### 事件钩子

插件可以挂钩到应用事件：

```lua
lc.on_event('EnterPost', function() ... end)   -- 进入目录后
lc.on_event('HoverPost', function() ... end)  -- 改变选中项后
```

#### UI 字符串扩展

插件系统向 Lua 字符串元表注入方法（`src/plugin/lc/ui.rs`）：

```lua
"filename".fg("blue")   -- 设置文本颜色
ansi_string:ansi()       -- 解析 ANSI 为 TUI Text
```

### Widget 系统（`src/widgets/`）

- **Renderable trait**：可渲染到终端的基 trait
  - `StatefulParagraph`：带滚动和滚动条的文本
  - `StatefulList`：带滚动的列表
- **FromLua trait**：从 Lua 值转换为 Rust widgets
  - 字符串 → `StatefulParagraph`
  - UserData（Text/Span）→ widgets
  - `PageEntry`：带有 `{key, display?, ...}` 的 Lua 表

### 键盘映射系统

键盘映射在 Lua 中配置，支持：

```lua
lc.keymap.set('main', 'd', function() ... end)        -- 单字符
lc.keymap.set('main', '<C-x>', function() ... end)     -- 功能键（角括号内）
lc.keymap.set('main', '<up>', 'scroll_by -1')          -- 方向键
lc.keymap.set('main', 'dd', function() ... end)         -- 多键序列
lc.keymap.set('main', '<C-x><C-c>', function() ... end) -- 多键序列
```

**规则**：
- 多字符键名必须用角括号：`<down>`, `<up>`, `<enter>`, `<tab>`, `<space>`, `<f5>` 等
- 单字符不需要角括号：`d`, `x`, `j`, `k`, `q` 等
- 角括号外的内容会被当作单独字符解析（如 `"down"` → d, o, w, n）
- 支持修饰符：`<C-x>` (Ctrl), `<A-k>` (Alt), `<S-tab>` (Shift)

Rust 通过**前缀匹配缓冲区**（`src/state.rs::tap_key()`）处理键盘事件：
1. 键盘事件累积在 `last_key_event_buffer` 中
2. 与注册的键盘映射匹配
3. 当只有一个键盘映射完全匹配时，执行其回调
4. 在无匹配或不匹配时清除缓冲区

### 状态管理

`src/state.rs` 中的 `State` 持有：
- `current_mode`：当前激活的模式（Main/Input）
- `current_path`：导航路径栈
- `current_page`：当前显示的 entries 列表
- `keymap_config`：注册的键盘绑定
- `last_key_event_buffer`：待处理的按键序列
- `current_preview`：预览面板内容（Renderable widget）

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

### 日志

- **Rust 日志**：`~/.local/state/lazycmd/lazycmd.log`（使用 tracing）
- **Lua 日志**：`~/.local/state/lazycmd/lua.log`（通过 `lc.log()`）

查看日志：
```bash
tail -f ~/.local/state/lazycmd/lazycmd.log
tail -f ~/.local/state/lazycmd/lua.log
```

## 内部命令

通过 `lc.cmd()` 发送内部命令：

| 命令 | 参数 | 用途 |
|------|------|------|
| `quit` | 无 | 退出应用 |
| `scroll_by <num>` | 可选数字 | 列表滚动指定行数（默认 1） |
| `scroll_preview_by <num>` | 可选数字 | 预览面板滚动指定行数（默认 1） |
| `reload` | 无 | 刷新当前列表，执行前调用 pre_reload 钩子 |
| `enter_filter_mode` | 无 | 进入过滤模式 |
| `exit_filter_mode` | 无 | 退出过滤模式 |
| `accept_filter` | 无 | 接受当前过滤 |
| `filter_clear` | 无 | 清除过滤条件 |

## 添加新功能

### 添加新的 LC API 函数

**重要**：每次修改或添加新的 LC API 函数后，必须同步更新 `preset/types.lua` 文件。

1. 在适当的 `src/plugin/lc/*.rs` 文件中添加函数
2. 在 `new_table()` 中注册
3. 如需状态访问，使用 `plugin::borrow_scope_state()` 或 `plugin::mut_scope_state()`
4. 如需触发更新，调用 `plugin::send_render_event()`
5. 在 `preset/types.lua` 中添加对应的类型定义

### 添加新的内部命令

1. 在 `src/app.rs::handle_command()` 的 match 分支中添加命令
2. 实现命令逻辑（修改 state、触发事件钩子等）
3. 如改变 UI，设置 `self.dirty = true`

### 添加新的事件钩子

1. 在 `src/events.rs` 中向 `EventHook` 枚举添加变体
2. 如需要，实现 `FromLua` trait
3. 在 `src/app.rs` 的适当位置调用 `self.run_event_hooks(...)`

### 创建新插件

1. 创建 `examples/plugins/myplugin.lazycmd/myplugin/init.lua`（debug）或 `~/.config/lazycmd/plugins/myplugin.lazycmd/myplugin/init.lua`（release）
2. 在 `examples/init.lua` 或 `~/.config/lazycmd/init.lua` 中添加插件配置

## 重要实现细节

### Lua 生命周期

- `Lua` 实例在 `App` 的生命周期内存活
- Lua 函数存储在 Rust 结构体中（如 `Keymap`、`EventHook` 回调）
- 调用 Lua 回调时，始终用 `plugin::scope()` 包装

### 错误处理

- 全程使用 `anyhow::Result<T>`
- 通过 `errors::install_hooks()` 安装 Panic 钩子
- Panic 处理器中的终端清理通过 `term::restore()` 进行

### 模式系统

目前只有 `Main` 模式完全实现。`Input` 模式存在但未连接。

### 预设文件加载

`src/plugin/lua.rs` 中的宏 `preset!($name)` 处理 debug（文件读取）和 release（嵌入字节）两种构建。

预设文件：
- `preset/util.lua` - 工具函数
- `preset/init.lua` - 默认初始化和键盘映射
- `preset/json.lua` - JSON 编解码
- `preset/inspect.lua` - 调试输出
- `preset/types.lua` - 类型定义

用户配置文件通过 `preset/init.lua` 末尾的 `require 'init'` 加载。

### 键盘映射解析

键盘映射字符串在 `src/keymap.rs` 中解析，支持单字符、角括号内的功能键、多键序列。单元测试位于 `src/keymap.rs` 的测试模块中。

### 附加说明

- **用户配置位置**：
  - Debug 模式：`examples/init.lua`, `examples/plugins/`
  - Release 模式：`~/.config/lazycmd/init.lua`, `~/.config/lazycmd/plugins/`

- **外部命令执行**：
  - `lc.system()` - 异步执行，通过回调获取结果
  - `lc.interactive()` - 执行交互式命令（有终端访问权限）
  - `lc.system.executable()` - 检查命令是否可执行（同步）

- **异步回调**：外部命令输出通过 `Event::LuaCallback` 发回，重新进入 Lua 作用域
