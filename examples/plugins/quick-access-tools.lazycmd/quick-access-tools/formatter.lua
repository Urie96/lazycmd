-- Formatter and data conversion tools

-- Check if a command is available
local function check_has(cmd)
  if not lc.system.executable(cmd) then error(cmd .. ' not found') end
end

-- Show error in preview
local function show_error(err) lc.api.page_set_preview(('Error: ' .. tostring(err)):fg 'red') end

local tools = {
  {
    key = 'json_format',
    display = 'JSON Format',
    description = 'Format JSON with indentation',
    converter = function(input, cb)
      local decoded = lc.json.decode(input)
      local encoded = lc.json.encode(decoded, { indent = 2 })
      cb(encoded, { language = 'json' })
    end,
  },
  {
    key = 'json_minify',
    display = 'JSON Minify',
    description = 'Minify JSON (remove whitespace)',
    converter = function(input, cb)
      lc.log('info', input)
      local decoded = lc.json.decode(input)
      local encoded = lc.json.encode(decoded)
      cb(encoded, { language = 'json' })
    end,
  },
  {
    key = 'stringify',
    display = 'Stringify',
    description = 'Convert text to JSON string',
    converter = function(input, cb)
      local encoded = lc.json.encode(input:trim())
      cb(encoded)
    end,
  },
  {
    key = 'unstringify',
    display = 'Unstringify',
    description = 'Convert JSON string to text',
    converter = function(input, cb)
      local decoded = lc.json.decode(input:trim())
      cb(decoded)
    end,
  },

  {
    key = 'base64_decode',
    display = 'Base64 Decode',
    description = 'Decode Base64 string',
    converter = function(input, cb)
      local decoded = lc.base64.decode(input:trim())
      cb(decoded)
    end,
  },
  {
    key = 'base64_encode',
    display = 'Base64 Encode',
    description = 'Encode string to Base64',
    converter = function(input, cb)
      local encoded = lc.base64.encode(input:trim())
      cb(encoded)
    end,
  },
  {
    key = 'url_encode',
    display = 'URL Encode',
    description = 'URL encode string',
    converter = function(input, cb)
      check_has 'python3'
      local script = "import sys; from urllib.parse import quote; print(quote(sys.stdin.read(), safe=''))"
      lc.system({ 'python3', '-c', script }, { stdin = input }, function(out)
        if out.code == 0 then
          cb(out.stdout)
        else
          show_error(out.stderr)
        end
      end)
    end,
  },
  {
    key = 'url_decode',
    display = 'URL Decode',
    description = 'URL decode string',
    converter = function(input, cb)
      check_has 'python3'
      local script = 'import sys; from urllib.parse import unquote; print(unquote(sys.stdin.read()))'
      lc.system({ 'python3', '-c', script }, { stdin = input }, function(out)
        if out.code == 0 then
          cb(out.stdout)
        else
          show_error(out.stderr)
        end
      end)
    end,
  },
  {
    key = 'json_to_nix',
    display = 'Convert JSON To Nix',
    description = 'Convert JSON to Nix expression',
    converter = function(input, cb)
      check_has 'nix'
      local tmp = lc.fs.tempfile { suffix = '.json', content = input }

      local expr = 'builtins.fromJSON (builtins.readFile "' .. tmp .. '")'
      lc.system({ 'nix-instantiate', '--eval', '--expr', expr }, function(out)
        lc.fs.remove(tmp)

        if out.code == 0 then
          cb(out.stdout, { language = 'nix' })
        else
          show_error(out.stderr)
        end
      end)
    end,
  },
  {
    key = 'yaml_to_json',
    display = 'Convert YAML To JSON',
    description = 'Convert YAML to JSON',
    converter = function(input, cb)
      local decoded = lc.yaml.decode(input)
      local json = lc.json.encode(decoded)
      cb(json, { language = 'json' })
    end,
  },
  {
    key = 'json_to_yaml',
    display = 'Convert JSON To YAML',
    description = 'Convert JSON to YAML',
    converter = function(input, cb)
      local decoded = lc.json.decode(input)
      local yaml = lc.yaml.encode(decoded)
      cb(yaml, { language = 'yaml' })
    end,
  },
}

-- Show result in preview
local function show_preview(result, opt)
  if opt and opt.language then
    lc.api.page_set_preview(lc.style.highlight(result, opt.language))
  else
    lc.api.page_set_preview(lc.style.text {
      lc.style.line {
        lc.style.span(result),
      },
    })
  end
end

-- Read from clipboard
local function read_clipboard(cb)
  local ok, content = pcall(lc.clipboard.get)
  if not ok then
    show_error(content)
  elseif #content == 0 then
    show_error 'Clipboard is empty'
  else
    cb(content)
  end
end

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

for _, tool in ipairs(tools) do
  if tool.converter then
    tool.on_enter = function()
      read_clipboard(function(content)
        local ok, err = pcall(tool.converter, content, show_preview)
        if not ok then show_error(err) end
      end)
    end

    tool.on_copy = function()
      read_clipboard(function(content)
        local ok, err = pcall(tool.converter, content, write_clipboard)
        if not ok then show_error(err) end
      end)
    end
  end
end

return tools
