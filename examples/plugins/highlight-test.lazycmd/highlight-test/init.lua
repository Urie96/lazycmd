local M = {}

-- 支持的语言和示例代码
local code_samples = {
  js = {
    name = 'JavaScript',
    extension = 'js',
    code = [[
function greet(name) {
  const message = `Hello, ${name}!`;
  console.log(message);
  return message;
}

class Calculator {
  constructor() {
    this.result = 0;
  }

  add(a, b) {
    return a + b;
  }
}

// Arrow functions
const double = (x) => x * 2;

// Async/await
async function fetchData(url) {
  try {
    const response = await fetch(url);
    const data = await response.json();
    return data;
  } catch (error) {
    console.error('Error:', error);
  }
}
]],
  },
  py = {
    name = 'Python',
    extension = 'py',
    code = [[
# A simple Python example
def greet(name):
    """Greet someone by name."""
    message = f"Hello, {name}!"
    print(message)
    return message

class Calculator:
    def __init__(self):
        self.result = 0

    def add(self, a, b):
        return a + b

# Lambda functions
double = lambda x: x * 2

# Async/await
import asyncio

async def fetch_data(url):
    try:
        response = await asyncio.get_event_loop().run_in_executor(
            None, lambda: requests.get(url)
        )
        return response.json()
    except Exception as e:
        print(f"Error: {e}")
]],
  },
  rs = {
    name = 'Rust',
    extension = 'rs',
    code = [[
// A simple Rust example
fn greet(name: &str) -> String {
    let message = format!("Hello, {}!", name);
    println!("{}", message);
    message
}

struct Calculator {
    result: i32,
}

impl Calculator {
    fn new() -> Self {
        Self { result: 0 }
    }

    fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}

// Closures
let double = |x: i32| x * 2;

// Async/await
async fn fetch_data(url: &str) -> Result<serde_json::Value, reqwest::Error> {
    match reqwest::get(url).await {
        Ok(response) => response.json().await,
        Err(e) => Err(e),
    }
}
]],
  },
  lua = {
    name = 'Lua',
    extension = 'lua',
    code = [[
-- A simple Lua example
function greet(name)
  local message = string.format("Hello, %s!", name)
  print(message)
  return message
end

local Calculator = {}
Calculator.__index = Calculator

function Calculator.new()
  return setmetatable({result = 0}, Calculator)
end

function Calculator:add(a, b)
  return a + b
end

-- Anonymous functions
local double = function(x)
  return x * 2
end

-- Table operations
local users = {
  {name = "Alice", age = 25},
  {name = "Bob", age = 30},
}

for i, user in ipairs(users) do
  print(string.format("%s: %d", user.name, user.age))
end
]],
  },
  go = {
    name = 'Go',
    extension = 'go',
    code = [[
// A simple Go example
package main

import "fmt"

func greet(name string) string {
    message := fmt.Sprintf("Hello, %s!", name)
    fmt.Println(message)
    return message
}

type Calculator struct {
    result int
}

func (c *Calculator) Add(a, b int) int {
    return a + b
}

func main() {
    // Anonymous function
    double := func(x int) int {
        return x * 2
    }

    fmt.Println(greet("World"))
    fmt.Println(double(5))
}
]],
  },
  java = {
    name = 'Java',
    extension = 'java',
    code = [[
// A simple Java example
public class HelloWorld {
    public static void main(String[] args) {
        System.out.println("Hello, World!");
    }

    public static String greet(String name) {
        String message = String.format("Hello, %s!", name);
        System.out.println(message);
        return message;
    }

    static class Calculator {
        private int result = 0;

        public int add(int a, int b) {
            return a + b;
        }
    }
}
]],
  },
  c = {
    name = 'C',
    extension = 'c',
    code = [[
// A simple C example
#include <stdio.h>
#include <string.h>

void greet(const char* name) {
    char message[100];
    snprintf(message, sizeof(message), "Hello, %s!", name);
    printf("%s\n", message);
}

typedef struct {
    int result;
} Calculator;

int calculator_add(const Calculator* c, int a, int b) {
    return a + b;
}

int main() {
    greet("World");
    return 0;
}
]],
  },
  cpp = {
    name = 'C++',
    extension = 'cpp',
    code = [[
// A simple C++ example
#include <iostream>
#include <string>
#include <vector>

void greet(const std::string& name) {
    std::string message = "Hello, " + name + "!";
    std::cout << message << std::endl;
}

class Calculator {
private:
    int result;
public:
    Calculator() : result(0) {}

    int add(int a, int b) {
        return a + b;
    }
};

int main() {
    greet("World");

    // Lambda expression
    auto double = [](int x) { return x * 2; };
    std::cout << double(5) << std::endl;

    return 0;
}
]],
  },
  sql = {
    name = 'SQL',
    extension = 'sql',
    code = [[
-- A simple SQL example
SELECT
    u.id,
    u.name,
    u.email,
    COUNT(o.id) as order_count,
    SUM(o.amount) as total_amount
FROM users u
LEFT JOIN orders o ON u.id = o.user_id
WHERE u.created_at >= '2024-01-01'
    AND u.status = 'active'
GROUP BY u.id, u.name, u.email
HAVING COUNT(o.id) > 0
ORDER BY total_amount DESC
LIMIT 10;

-- Insert example
INSERT INTO products (name, price, category_id)
VALUES ('New Product', 99.99, 1)
ON CONFLICT (name) DO UPDATE
SET price = EXCLUDED.price
WHERE products.price < EXCLUDED.price;
]],
  },
  html = {
    name = 'HTML',
    extension = 'html',
    code = [[
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Hello World</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            margin: 0;
            padding: 20px;
        }
        .container {
            max-width: 800px;
            margin: 0 auto;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Hello, World!</h1>
        <p>This is a sample HTML document.</p>
        <button id="clickMe">Click Me</button>
    </div>
    <script>
        document.getElementById('clickMe').addEventListener('click', function() {
            alert('Button clicked!');
        });
    </script>
</body>
</html>
]],
  },
  css = {
    name = 'CSS',
    extension = 'css',
    code = [[
/* CSS Example */
body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
    background-color: #f5f5f5;
    margin: 0;
    padding: 0;
}

.container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
}

.card {
    background: white;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    padding: 20px;
    margin-bottom: 20px;
}

.btn {
    display: inline-block;
    padding: 10px 20px;
    background-color: #007bff;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    transition: background-color 0.3s;
}

.btn:hover {
    background-color: #0056b3;
}

@media (max-width: 768px) {
    .container {
        padding: 10px;
    }
}
]],
  },
  json = {
    name = 'JSON',
    extension = 'json',
    code = [[
{
  "name": "highlight-test",
  "version": "1.0.0",
  "description": "Test syntax highlighting",
  "main": "init.lua",
  "languages": [
    "javascript",
    "python",
    "rust",
    "lua",
    "go",
    "java",
    "c",
    "cpp",
    "sql",
    "html",
    "css",
    "json"
  ],
  "config": {
    "theme": "dark",
    "fontSize": 14,
    "showLineNumbers": true
  },
  "features": {
    "syntaxHighlighting": true,
    "autoDetect": true,
    "customThemes": false
  }
}
]],
  },
  yaml = {
    name = 'YAML',
    extension = 'yaml',
    code = [[
# YAML Configuration
name: highlight-test
version: 1.0.0
description: Test syntax highlighting

languages:
  - javascript
  - python
  - rust
  - lua

config:
  theme: dark
  fontSize: 14
  showLineNumbers: true

features:
  syntaxHighlighting: true
  autoDetect: true
  customThemes: false

servers:
  production:
    host: example.com
    port: 443
    ssl: true
  development:
    host: localhost
    port: 8080
    ssl: false
]],
  },
  bash = {
    name = 'Bash',
    extension = 'sh',
    code = [==[
#!/bin/bash

# Bash Script Example
set -e

NAME="${1:-World}"

greet() {
    echo "Hello, ${NAME}!"
}

# Calculate sum
calculate_sum() {
    local a=$1
    local b=$2
    echo $((a + b))
}

# Loop through files
process_files() {
    for file in /tmp/*.txt; do
        if [[ -f "$file" ]]; then
            echo "Processing: $file"
            cat "$file"
        fi
    done
}

# Main execution
greet
echo "Sum of 5 and 3: $(calculate_sum 5 3)"

# Check command exists
if command -v git &> /dev/null; then
    echo "Git is installed"
fi
]==],
  },
}

-- 高亮代码字符串
local function highlight_code(lang)
  local sample = code_samples[lang]
  if not sample then
    lc.notify('Unknown language: ' .. lang)
    return
  end

  local highlighted = lc.style.highlight(sample.code, lang)
  lc.api.page_set_preview(highlighted)
  lc.notify('Highlighted: ' .. sample.name)
end

function M.setup()
  -- 键盘绑定 - 单键快捷键
  lc.keymap.set('main', '1', function() highlight_code 'js' end)
  lc.keymap.set('main', '2', function() highlight_code 'py' end)
  lc.keymap.set('main', '3', function() highlight_code 'rs' end)
  lc.keymap.set('main', '4', function() highlight_code 'lua' end)
  lc.keymap.set('main', '5', function() highlight_code 'go' end)
  lc.keymap.set('main', '6', function() highlight_code 'java' end)
  lc.keymap.set('main', '7', function() highlight_code 'c' end)
  lc.keymap.set('main', '8', function() highlight_code 'cpp' end)
  lc.keymap.set('main', '9', function() highlight_code 'sql' end)
  lc.keymap.set('main', '0', function() highlight_code 'html' end)
  lc.keymap.set('main', 'q', function() highlight_code 'css' end)
  lc.keymap.set('main', 'w', function() highlight_code 'json' end)
  lc.keymap.set('main', 'e', function() highlight_code 'yaml' end)
  lc.keymap.set('main', 'r', function() highlight_code 'bash' end)

  -- 显示帮助
  lc.keymap.set('main', '?', function()
    local help_text = [[
Syntax Highlight Test - Keybindings:

1-0, q-r - Highlight sample code:
  1: JavaScript    6: Java       q: CSS
  2: Python        7: C          w: JSON
  3: Rust          8: C++        e: YAML
  4: Lua           9: SQL        r: Bash
  5: Go            0: HTML

l - List all available languages in log file

Use arrow keys/j/k to navigate, Enter to preview
]]
    lc.api.page_set_preview(help_text)
  end)

  -- 列出所有支持的语言
  lc.keymap.set('main', 'l', function()
    lc.log('info', 'Listing all supported languages:')
    -- 常见语言列表
    local common_langs = {
      'javascript',
      'typescript',
      'python',
      'rust',
      'go',
      'java',
      'c',
      'cpp',
      'bash',
      'sh',
      'zsh',
      'fish',
      'powershell',
      'lua',
      'ruby',
      'php',
      'perl',
      'html',
      'css',
      'json',
      'yaml',
      'toml',
      'sql',
      'markdown',
      'dockerfile',
      'kotlin',
      'swift',
      'scala',
      'haskell',
      'elixir',
      'erlang',
      'clojure',
      'racket',
      'cmake',
      'makefile',
      'dockerfile',
      'nginx',
      'apache',
      'ini',
    }

    -- 尝试不同的语言名称变体
    local lang_variants = {
      bash = { 'bash', 'sh', 'shell' },
      c = { 'c', 'ansi-c' },
      cpp = { 'c++', 'cpp', 'cxx' },
      powershell = { 'powershell', 'ps1', 'posh' },
    }

    local function test_lang(lang_name)
      -- 测试基本名称
      local test_code = code_samples[lang_name] and code_samples[lang_name].code or 'test code'
      local ok, err = pcall(lc.style.highlight, test_code, lang_name)

      if ok then
        lc.log('info', '✓ {}', lang_name)
        return true
      else
        -- 尝试变体
        if lang_variants[lang_name] then
          for _, variant in ipairs(lang_variants[lang_name]) do
            if variant ~= lang_name then
              ok, err = pcall(lc.style.highlight, test_code, variant)
              if ok then
                lc.log('info', "✓ {} (use '{}')", lang_name, variant)
                return true
              end
            end
          end
        end
        lc.log('warn', '✗ {} - {}', lang_name, tostring(err):sub(1, 100))
        return false
      end
    end

    for _, lang in ipairs(common_langs) do
      test_lang(lang)
    end

    lc.notify 'Language list written to log file (~/.local/state/lazycmd/lua.log)'
    lc.notify 'Highlight debug log: ~/.local/state/lazycmd/highlight.log'
  end)

  -- 测试特定语言的不同变体
  lc.keymap.set('main', 'L', function()
    lc.log('info', 'Testing bash language variants:')
    local bash_test = [[#!/bin/bash
echo "Hello World"]]
    local variants = { 'bash', 'sh', 'shell', 'Bash', 'BASH', 'shellscript' }
    for _, v in ipairs(variants) do
      local ok, err = pcall(lc.style.highlight, bash_test, v)
      lc.log('info', "bash variant '{}': {}", v, ok and 'OK' or 'FAIL: ' .. tostring(err):sub(1, 50))
    end

    lc.log('info', 'Testing C language variants:')
    local c_test = [[int main() { return 0; }]]
    variants = { 'c', 'C', 'ansi-c', 'ansi_c' }
    for _, v in ipairs(variants) do
      local ok, err = pcall(lc.style.highlight, c_test, v)
      lc.log('info', "C variant '{}': {}", v, ok and 'OK' or 'FAIL: ' .. tostring(err):sub(1, 50))
    end

    lc.notify 'Language variant test complete - check log files'
  end)

  -- 初始显示帮助
  lc.keymap.set('main', 'h', function() lc.notify 'Press ? for help' end)

  lc.notify 'Highlight test loaded! Press ? for help, or use arrow keys/Enter'
end

function M.list(path, cb)
  local entries = {}
  for lang, sample in pairs(code_samples) do
    table.insert(entries, {
      key = lang,
      display = string.format('%-15s .%s', sample.name, sample.extension),
    })
  end
  -- 排序
  table.sort(entries, function(a, b) return a.key < b.key end)
  cb(entries)
end

function M.preview(entry, cb)
  local sample = code_samples[entry.key]
  if sample then
    local highlighted = lc.style.highlight(sample.code, entry.key)
    cb(highlighted)
  else
    cb 'No preview available'
  end
end

return M
