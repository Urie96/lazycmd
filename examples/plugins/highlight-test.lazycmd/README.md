# Highlight Test Plugin

用于测试 lazycmd 语法高亮功能的插件。

## 功能特性

- 支持 13 种编程语言的语法高亮
- 内置各语言的示例代码
- 键盘快捷键快速切换
- 列表导航和实时预览

## 支持的语言

| 按键 | 语言 | 扩展名 |
|------|------|--------|
| 1 | JavaScript | .js |
| 2 | Python | .py |
| 3 | Rust | .rs |
| 4 | Lua | .lua |
| 5 | Go | .go |
| 6 | Java | .java |
| 7 | C | .c |
| 8 | C++ | .cpp |
| 9 | SQL | .sql |
| 0 | HTML | .html |
| q | CSS | .css |
| w | JSON | .json |
| e | YAML | .yaml |
| r | Bash | .sh |

## 使用方法

### 运行插件

```bash
cargo run -- highlight-test
```

### 快捷键

- **数字键 1-0, q, w, e, r** - 高亮对应语言的示例代码
- **?** - 显示帮助信息
- **方向键 / j/k** - 导航语言列表
- **Enter** - 查看选中语言的代码预览

### API 用法示例

```lua
-- 高亮 JavaScript 代码
local code = [[
function greet(name) {
  console.log("Hello, " + name);
}
]]

local highlighted = lc.style.highlight(code, "javascript")
lc.api.page_set_preview(highlighted)

-- 高亮 Python 代码
local python_code = [[
def greet(name):
    return f"Hello, {name}!"
]]

local highlighted_py = lc.style.highlight(python_code, "python")
lc.api.page_set_preview(highlighted_py)
```

## 支持更多语言

syntect 支持超过 180 种编程语言。完整的支持列表请参考：
https://github.com/sublimehq/Packages

常用语言标识符：
- JavaScript/TypeScript: `javascript`, `typescript`
- Python: `python`
- Rust: `rust`
- Go: `go`
- Java: `java`
- C/C++: `c`, `cpp`
- Shell: `bash`, `sh`, `zsh`
- Web: `html`, `css`, `javascript`
- 配置文件: `json`, `yaml`, `toml`, `ini`
- 其他: `markdown`, `dockerfile`, `sql`, `php`, `ruby` 等
