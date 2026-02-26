# lazycmd AGENTS.md

本文档为在此 Rust 终端 UI 应用仓库中工作的 AI 实例提供项目特定信息。

## 插件开发工作流

lazycmd 的核心是其 Lua 插件系统。插件在启动时从 `preset/` 目录加载。

### 调试预设 Lua 文件

在 debug 构建（`cargo build`）中，Lua 预设文件直接从磁盘读取，允许实时编辑：

1. 运行 `cargo run -- process`
2. 编辑 `preset/` 中的文件（如 `preset/init.lua`）
3. 重启应用以查看更改

在 release 构建（`cargo run --release`）中，预设文件在编译时使用 `include_bytes!` 嵌入。

### 插件加载流程

1. `src/plugin/lua.rs::init_lua()` 在 `App::new()` 期间被调用
2. 加载 `preset/init.lua` 设置全局 `lc` API
3. 可通过 `require()` 加载其他插件 - 参见 `preset/plugins/process.lazycmd/`

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

| Lua 模块      | Rust 源文件               | 用途                                            |
| ------------- | ------------------------- | ----------------------------------------------- |
| `lc.api`      | `src/plugin/lc/api.rs`    | 页面管理（entries、预览、导航）、命令行参数访问 |
| `lc.fs`       | `src/plugin/lc/fs.rs`     | 同步文件系统操作                                |
| `lc.keymap`   | `src/plugin/lc/keymap.rs` | 注册键盘快捷键                                  |
| `lc.path`     | `src/plugin/lc/path.rs`   | 路径操作（split/join）                          |
| `lc.defer_fn` | `src/plugin/lc/mod.rs`    | 调度异步 Lua 回调                               |
| `lc.system`   | `src/plugin/lc/mod.rs`    | 异步执行外部命令                                |
| `lc.on_event` | `src/plugin/lc/mod.rs`    | 注册事件钩子                                    |
| `lc.cmd`      | `src/plugin/lc/mod.rs`    | 向 Rust 发送内部命令                            |

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
  - `PageEntry`：带有 `{key, display?, is_dir?, ...}` 的 Lua 表

### 键盘映射系统

键盘映射在 Lua 中配置并注册到 Rust：

```lua
lc.keymap.set('main', 'ctrl-j', function() ... end)
lc.keymap.set('main', 'down', 'scroll_by 1')  -- 命令字符串简写
```

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
│ Footer (height: 1) - empty     │
└─────────────────────────────────┘
```

## 插件示例：进程查看器

`preset/plugins/process.lazycmd/` 演示了一个完整的插件：

1. **列出**：使用 `lc.system({ 'ps', '-eo', 'pid,command' }, ...)` 获取进程列表
2. **预览**：使用 `lc.system({ 'pstree', '-p', pid }, ...)` 显示进程树
3. **集成**：从 `preset/init.lua` 事件钩子调用

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
```

### 内部命令

通过 `lc.cmd()` 可以发送内部命令到 Rust 端处理：

| 命令 | 参数 | 用途 |
|------|------|------|
| `quit` | 无 | 退出应用 |
| `scroll_by <num>` | 可选数字 | 列表滚动指定行数（默认 1） |
| `scroll_preview_by <num>` | 可选数字 | 预览面板滚动指定行数（默认 1） |
| `reload` | 无 | 刷新当前列表（重新调用插件的 `list()` 函数） |

**示例**：

```lua
-- 使用命令字符串快捷方式
lc.keymap.set('main', 'j', 'scroll_by 1')
lc.keymap.set('main', 'k', 'scroll_by -1')

-- 使用 reload 命令
lc.keymap.set('main', 'ctrl-r', 'reload')

-- 自定义刷新按键
map('main', 'f5', 'reload')
```

## 日志

日志使用 `tracing` crate 写入 `~/.local/state/lazycmd/lazycmd.log`：

```bash
# 实时查看日志
tail -f ~/.local/state/lazycmd/lazycmd.log
```

## 添加新功能

### 添加新的 LC API 函数

1. 在适当的 `src/plugin/lc/*.rs` 文件中添加函数
2. 在 `new_table()` 中注册它以添加到 lc 全局
3. 如果需要状态访问，使用 `plugin::borrow_scope_state()` 或 `plugin::mut_scope_state()`
4. 如果需要触发更新，调用 `plugin::send_render_event()`

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

1. 在 `preset/plugins/myplugin.lazycmd/` 下创建目录
2. 添加 `init.lua` 返回带有函数的表
3. 在 `preset/init.lua` 中更新 `package.path` 以包含你的插件目录
4. 在 `preset/init.lua` 或其他插件中使用 `require 'myplugin'`

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

`src/plugin/lua.rs` 中的宏 `preset!($name)` 处理 debug（文件读取）和 release（嵌入字节）两种构建。添加新预设文件时，确保将它们放置在 `preset/` 目录中。

### 键盘映射解析

键盘映射字符串在 `src/keymap.rs` 中解析，支持以下格式：

- 简单按键：`"down"`, `"up"`, `"q"` 等
- 带修饰符的按键：`"ctrl-d"`, `"alt-a"`, `"shift-b"` 等
- 支持的修饰符前缀：`ctrl-`, `alt-`, `shift-`

**重要**：当使用单字符按键（如 `"d"`）配合修饰符时，解析函数会保留修饰符。例如 `ctrl-d` 会生成 `KeyEvent { code: Char('d'), modifiers: CONTROL }`。键盘映射的单元测试位于 `src/keymap.rs` 的测试模块中。
