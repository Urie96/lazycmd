# lazycmd

一个基于 Rust + Lua 的终端 UI (TUI) 文件管理器/命令面板，灵感来源于 [yazi](https://github.com/sxyazi/yazi)。

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)

## 特性

- 🚀 **高性能** - 基于 Rust 构建，异步事件驱动
- 🔌 **Lua 插件系统** - 使用 Lua 5.4 脚本语言扩展功能
- 🎨 **语法高亮** - 内置 180+ 种编程语言语法高亮支持
- 🖥️ **现代化 UI** - 使用 ratatui 构建的美观终端界面
- ⌨️ **Vim 风格导航** - 支持 `j`/`k` 上下移动、`gg`/`G` 跳转、`/` 过滤等
- 💾 **页面缓存** - 目录导航时保持状态和滚动位置

## 预览

```
docker:/container
╭────────────────────────────────┬───────────────────────────────╮
│ intelligent_benz redis:alpine  │🆔 ID:         15eb56799f61    │
│myalpine         alpine       │📊 State:      running         │
│ naughty_allen    alpine        │ℹ️ Status:     Up 23 hours     │
│ pedantic_jones   alpine        │⌨️ Command:    sh -c echo      │
│ silly_khayyam    alpine        │'容器启动了' && tail -f        │
│ dreamy_mcnulty   alpine        │/dev/null                      │
│                                │🚪 Entrypoint:                 │
│                                │📅 Created:    2026-03-07      │
│                                │15:45:18 +0800 CST             │
│                                │                               │
│                                │Logs:                          │
│                                │容器启动了                     │
│                                │容器启动了                     │
│                                │                               │
╰────────────────────────────────┴───────────────────────────────╯
```

## 安装

### 从源码编译

```bash
# 克隆仓库
git clone https://github.com/urie/lazycmd.git
cd lazycmd

# 构建
cargo build --release

# 运行
cargo run --release -- <plugin_name>
```

## 项目结构

```
lazycmd/
├── src/                    # Rust 源代码
│   ├── main.rs            # 入口点
│   ├── app.rs             # 主应用逻辑
│   ├── state.rs           # 状态管理
│   ├── events.rs          # 事件系统
│   ├── keymap.rs          # 键盘映射
│   ├── plugin/            # Lua 插件系统
│   └── widgets/           # UI 组件
├── preset/                # 预设文件
│   ├── lua/              # Lua 预设脚本
│   ├── syntaxes/         # 语法定义文件
│   └── themes/           # 颜色主题
└── examples/             # 示例插件和配置
    ├── init.lua          # 用户配置
    └── plugins/          # 示例插件
```

## 核心概念

### 插件系统

lazycmd 的核心功能通过 Lua 插件实现。每个插件是一个包含 `init.lua` 的目录：

```
examples/plugins/myplugin.lazycmd/
└── myplugin/
    └── init.lua
```

插件需要导出以下函数：

```lua
-- init.lua
local M = {}

-- 初始化函数（可选）
function M.setup()
    -- 设置键盘映射等
end

-- 列出条目（必需）
function M.list(path, cb)
    -- 获取条目列表
    -- cb(entries) 回调传递结果
end

-- 预览条目（可选）
function M.preview(entry, cb)
    -- 设置预览内容
    -- cb(preview_widget) 回调传递预览
end

return M
```

### PageEntry 格式

```lua
{
    key = "item_name",           -- 必需：唯一标识
    display = "显示文本",         -- 可选：显示文本，默认使用 key
    -- 自定义字段...
}
```

### LC API

全局表 `lc` 提供丰富的 API：

| 模块 | 功能 |
|------|------|
| `lc.api` | 页面管理、导航 |
| `lc.fs` | 文件系统操作 |
| `lc.http` | HTTP 请求 |
| `lc.system` | 执行外部命令 |
| `lc.cache` | 持久化缓存 |
| `lc.time` | 时间解析格式化 |
| `lc.style` | UI 样式和语法高亮 |
| `lc.keymap` | 键盘映射 |
| `lc.json` | JSON 编解码 |
| `lc.cmd` | 发送内部命令 |

## 内置插件

lazycmd 自带多个示例插件：

| 插件 | 说明 |
|------|------|
| `process` | 进程管理器 |
| `memos` | Memos 笔记客户端 |
| `himalaya` | 邮件客户端 |
| `systemd` | systemd 服务管理 |
| `docker` | Docker 容器管理 |

## 键盘快捷键

### 主模式

| 按键 | 功能 |
|------|------|
| `↑` / `↓` / `j` / `k` | 上下移动 |
| `gg` | 跳到开头 |
| `G` | 跳到结尾 |
| `/` | 进入过滤模式 |
| `Enter` / `→` | 进入目录 |
| `←` | 返回上级 |
| `q` | 退出 |
| `Ctrl+r` | 刷新 |

### 过滤模式

| 按键 | 功能 |
|------|------|
| 任意字符 | 输入过滤文本 |
| `Enter` | 应用过滤 |
| `Esc` | 退出过滤模式 |
| `Ctrl+u` | 清空过滤 |

## 配置

在 `examples/init.lua` (debug) 或 `~/.config/lazycmd/init.lua` (release) 中配置：

```lua
lc.config {
  plugins = {
    {
      'myplugin',
      config = function()
        require('myplugin').setup {
          option = value,
        }
      end,
    },
  },
}
```

## 文档

- [源码指南](src/README.md) - Rust 核心代码说明
- [插件系统](src/plugin/README.md) - Lua API 详细文档
- [预设脚本](preset/lua/README.md) - Lua 预设模块说明
- [UI 组件](src/widgets/README.md) - widgets 模块说明

## 依赖

### Rust 核心依赖

- **mlua** - Lua 5.4 绑定
- **tokio** - 异步运行时
- **crossterm** - 终端控制
- **ratatui** - TUI 组件库
- **syntect** - 语法高亮
- **reqwest** - HTTP 客户端
- **chrono** - 时间处理

## 开发

### 调试模式

Debug 模式下：
- 预设文件从 `preset/lua/` 读取
- 用户配置从 `examples/` 读取

### Release 模式

Release 模式下：
- 预设文件嵌入二进制
- 用户配置从 `~/.config/lazycmd/` 读取

### 日志

```bash
# 查看 Rust 日志
tail -f ~/.local/state/lazycmd/lazycmd.log

# 查看 Lua 日志
tail -f ~/.local/state/lazycmd/lua.log
```

## 贡献

欢迎提交 Issue 和 Pull Request！
