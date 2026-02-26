-- 测试 lc.api.argv() 的示例代码
-- 可以在 preset/init.lua 中添加此代码来测试功能

-- 示例 1: 基本使用
local args = lc.api.argv()
lc.notify("命令行参数: " .. lc.inspect(args))

-- 示例 2: 检查特定参数是否存在
local function has_arg(arg_name)
  for _, arg in ipairs(lc.api.argv()) do
    if arg == arg_name then
      return true
    end
  end
  return false
end

-- 示例 3: 解析键值对参数 (如 --config=file.conf)
local function parse_kv_args(args)
  local result = {}
  for _, arg in ipairs(args) do
    local key, value = arg:match("^%-%-(.+)=(.+)$")
    if key and value then
      result[key] = value
    end
  end
  return result
end
