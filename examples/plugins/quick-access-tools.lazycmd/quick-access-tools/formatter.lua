-- Formatter and data conversion tools

-- Check if a command is available
local function has_cmd(cmd) return lc.system.executable(cmd) end

-- Read from clipboard
local function read_clipboard()
  local ok, content = pcall(lc.clipboard.get)
  if ok then
    return content
  else
    return nil, content
  end
end

-- Show error in preview
local function show_error(err) lc.api.page_set_preview(('Error: ' .. tostring(err)):fg 'red') end

-- Write to clipboard
local function write_clipboard(text)
  local ok, err = pcall(lc.clipboard.set, text)
  if ok then
    lc.notify 'Copied to clipboard'
  else
    show_error('Failed to copy to clipboard: ' .. tostring(err))
  end
  return ok, err
end

-- Show result in preview
local function show_preview(result, language)
  if language then
    lc.api.page_set_preview(lc.style.highlight(result, language))
  else
    lc.api.page_set_preview(lc.style.text {
      lc.style.line {
        lc.style.span(result),
      },
    })
  end
end

-- Tool implementations
local function json_format(cb)
  local input, err = read_clipboard()
  if err or not input or #input == 0 then
    show_error(err or 'Clipboard is empty')
    return
  end
  local ok, decoded = pcall(lc.json.decode, input)
  if not ok then
    show_error('Invalid JSON: ' .. tostring(decoded))
    return
  end
  local encoded = lc.json.encode(decoded, { indent = 2 })
  cb(encoded, 'json')
end

local function json_minify(cb)
  local input, err = read_clipboard()
  if err or not input or #input == 0 then
    show_error(err or 'Clipboard is empty')
    return
  end
  local ok, decoded = pcall(lc.json.decode, input)
  if not ok then
    show_error('Invalid JSON: ' .. tostring(decoded))
    return
  end
  local encoded = lc.json.encode(decoded)
  cb(encoded, 'json')
end

local function stringify(cb)
  local input, err = read_clipboard()
  if err or not input or #input == 0 then
    show_error(err or 'Clipboard is empty')
    return
  end
  local encoded = lc.json.encode(input:trim())
  cb(encoded)
end

local function unstringify(cb)
  local input, err = read_clipboard()
  if err or not input or #input == 0 then
    show_error(err or 'Clipboard is empty')
    return
  end
  local ok, decoded = pcall(lc.json.decode, input:trim())
  if not ok then
    show_error('Invalid JSON string: ' .. tostring(decoded))
    return
  end
  if type(decoded) ~= 'string' then
    show_error 'Input is not a JSON string'
    return
  end
  cb(decoded)
end

local function base64_decode(cb)
  local input, err = read_clipboard()
  if err or not input or #input == 0 then
    show_error(err or 'Clipboard is empty')
    return
  end
  local ok, decoded = pcall(lc.base64.decode, input:trim())
  if not ok then
    show_error('Invalid Base64: ' .. tostring(decoded))
    return
  end
  cb(decoded)
end

local function base64_encode(cb)
  local input, err = read_clipboard()
  if err or not input or #input == 0 then
    show_error(err or 'Clipboard is empty')
    return
  end
  local encoded = lc.base64.encode(input:trim())
  cb(encoded)
end

local function url_encode(cb)
  if not has_cmd 'python3' then
    show_error 'python3 not found'
    return
  end
  local input, err = read_clipboard()
  if err or not input or #input == 0 then
    show_error(err or 'Clipboard is empty')
    return
  end
  local script = "import sys; from urllib.parse import quote; print(quote(sys.stdin.read(), safe=''))"
  lc.system({ 'python3', '-c', script }, { stdin = input }, function(out)
    if out.code == 0 then
      cb(out.stdout)
    else
      show_error(out.stderr)
    end
  end)
end

local function url_decode(cb)
  if not has_cmd 'python3' then
    show_error 'python3 not found'
    return
  end
  local input, err = read_clipboard()
  if err or not input or #input == 0 then
    show_error(err or 'Clipboard is empty')
    return
  end
  local script = 'import sys; from urllib.parse import unquote; print(unquote(sys.stdin.read()))'
  lc.system({ 'python3', '-c', script }, { stdin = input }, function(out)
    if out.code == 0 then
      cb(out.stdout)
    else
      show_error(out.stderr)
    end
  end)
end

local function json_to_nix(cb)
  local input, err = read_clipboard()
  if err or not input or #input == 0 then
    show_error(err or 'Clipboard is empty')
    return
  end
  local tmp = '/tmp/lazycmd_t_json.json'
  lc.fs.write_file_sync(tmp, input, function(err)
    if err then
      show_error(err)
      return
    end

    local expr = 'builtins.fromJSON (builtins.readFile "' .. tmp .. '")'
    lc.system({ 'nix', 'eval', '--impure', '--expr', expr }, function(out)
      lc.system({ 'rm', tmp }, function() end)

      if out.code == 0 then
        cb(out.stdout, 'nix')
      else
        show_error(out.stderr)
      end
    end)
  end)
end

local function yaml_to_nix(cb)
  if not has_cmd 'yq' then
    show_error 'yq not found'
    return
  end
  local input, err = read_clipboard()
  if err or not input or #input == 0 then
    show_error(err or 'Clipboard is empty')
    return
  end
  lc.system({ 'yq', '-o=json', '.' }, { stdin = input }, function(out)
    if out.code ~= 0 then
      show_error(out.stderr)
      return
    end

    local tmp = '/tmp/lazycmd_t_yaml.json'
    lc.fs.write_file_sync(tmp, out.stdout, function(err)
      if err then
        show_error(err)
        return
      end

      local expr = 'builtins.fromJSON (builtins.readFile "' .. tmp .. '")'
      lc.system({ 'nix', 'eval', '--impure', '--expr', expr }, function(out)
        lc.system({ 'rm', tmp }, function() end)

        if out.code == 0 then
          cb(out.stdout, 'nix')
        else
          show_error(out.stderr)
        end
      end)
    end)
  end)
end

return {
  {
    key = 'json_format',
    display = 'JSON Format',
    description = 'Format JSON with indentation',
    on_enter = function() json_format(show_preview) end,
    on_copy = function() json_format(write_clipboard) end,
  },

  {
    key = 'json_minify',
    display = 'JSON Minify',
    description = 'Minify JSON (remove whitespace)',
    on_enter = function() json_minify(show_preview) end,
    on_copy = function() json_minify(write_clipboard) end,
  },

  {
    key = 'stringify',
    display = 'Stringify',
    description = 'Convert text to JSON string',
    on_enter = function() stringify(show_preview) end,
    on_copy = function() stringify(write_clipboard) end,
  },

  {
    key = 'unstringify',
    display = 'Unstringify',
    description = 'Convert JSON string to text',
    on_enter = function() unstringify(show_preview) end,
    on_copy = function() unstringify(write_clipboard) end,
  },

  {
    key = 'base64_decode',
    display = 'Base64 Decode',
    description = 'Decode Base64 string',
    on_enter = function() base64_decode(show_preview) end,
    on_copy = function() base64_decode(write_clipboard) end,
  },

  {
    key = 'base64_encode',
    display = 'Base64 Encode',
    description = 'Encode string to Base64',
    on_enter = function() base64_encode(show_preview) end,
    on_copy = function() base64_encode(write_clipboard) end,
  },

  {
    key = 'url_encode',
    display = 'URL Encode',
    description = 'URL encode string',
    on_enter = function() url_encode(show_preview) end,
    on_copy = function() url_encode(write_clipboard) end,
  },

  {
    key = 'url_decode',
    display = 'URL Decode',
    description = 'URL decode string',
    on_enter = function() url_decode(show_preview) end,
    on_copy = function() url_decode(write_clipboard) end,
  },

  {
    key = 'json_to_nix',
    display = 'Convert JSON To Nix',
    description = 'Convert JSON to Nix expression',
    on_enter = function() json_to_nix(show_preview) end,
    on_copy = function() json_to_nix(write_clipboard) end,
  },

  {
    key = 'yaml_to_nix',
    display = 'Convert YAML To Nix',
    description = 'Convert YAML to Nix expression',
    on_enter = function() yaml_to_nix(show_preview) end,
    on_copy = function() yaml_to_nix(write_clipboard) end,
  },
}
