-- t: A lazycmd plugin for text transformations and utilities

local M = {}

-- List of available tools
local tools = {
  { key = "json_format", display = "JSON Format" },
  { key = "json_minify", display = "JSON Minify" },
  { key = "stringify", display = "Stringify" },
  { key = "unstringify", display = "Unstringify" },
  { key = "base64_decode", display = "Base64 Decode" },
  { key = "base64_encode", display = "Base64 Encode" },
  { key = "url_encode", display = "URL Encode" },
  { key = "url_decode", display = "URL Decode" },
  { key = "unix_to_date", display = "Unix Timestamp To Date" },
  { key = "json_to_nix", display = "Convert JSON To Nix" },
  { key = "yaml_to_nix", display = "Convert YAML To Nix" },
  { key = "toml_to_nix", display = "Convert TOML To Nix" },
}

-- Check if a command is available
local function has_cmd(cmd)
  return lc.system.executable(cmd)
end

-- Read from clipboard
local function read_clipboard(cb)
  local cmd = nil
  if has_cmd("pbpaste") then
    cmd = { "pbpaste" }
  elseif has_cmd("xclip") then
    cmd = { "xclip", "-selection", "clipboard", "-o" }
  elseif has_cmd("wl-paste") then
    cmd = { "wl-paste" }
  end

  if not cmd then
    cb(nil, "No clipboard command available")
    return
  end

  lc.system(cmd, function(out)
    if out.code == 0 then
      cb(out.stdout, nil)
    else
      cb(nil, out.stderr)
    end
  end)
end

-- Write to clipboard
local function write_clipboard(text, cb)
  local cmd = nil
  if has_cmd("pbcopy") then
    cmd = { "pbcopy" }
  elseif has_cmd("xclip") then
    cmd = { "xclip", "-selection", "clipboard" }
  elseif has_cmd("wl-copy") then
    cmd = { "wl-copy" }
  end

  if not cmd then
    if cb then cb(false, "No clipboard command available") end
    return
  end

  lc.system(cmd, { stdin = text }, function(out)
    if cb then
      cb(out.code == 0, out.code ~= 0 and out.stderr or nil)
    end
  end)
end

-- Tool implementations
local tools_impl = {
  json_format = function(input, cb)
    -- Use lc.json.encode with indent for formatting
    local ok, decoded = pcall(lc.json.decode, input)
    if not ok then
      cb(nil, "Invalid JSON: " .. tostring(decoded))
      return
    end
    local encoded = lc.json.encode(decoded, { indent = 2 })
    cb(encoded, nil)
  end,

  json_minify = function(input, cb)
    -- Use lc.json.encode without indent for minifying
    local ok, decoded = pcall(lc.json.decode, input)
    if not ok then
      cb(nil, "Invalid JSON: " .. tostring(decoded))
      return
    end
    local encoded = lc.json.encode(decoded)
    cb(encoded, nil)
  end,

  stringify = function(input, cb)
    -- Convert plain text to JSON string (escape and wrap in quotes)
    local trimmed = string.gsub(input, "^%s+", ""):gsub("%s+$", "")
    local encoded = lc.json.encode(trimmed)
    cb(encoded, nil)
  end,

  unstringify = function(input, cb)
    -- Convert JSON string to plain text
    local trimmed = string.gsub(input, "^%s+", ""):gsub("%s+$", "")
    local ok, decoded = pcall(lc.json.decode, trimmed)
    if not ok then
      cb(nil, "Invalid JSON string: " .. tostring(decoded))
      return
    end
    if type(decoded) ~= "string" then
      cb(nil, "Input is not a JSON string")
      return
    end
    cb(decoded, nil)
  end,

  base64_decode = function(input, cb)
    local trimmed = string.gsub(input, "^%s+", ""):gsub("%s+$", "")
    local ok, decoded = pcall(lc.base64.decode, trimmed)
    if not ok then
      cb(nil, "Invalid Base64: " .. tostring(decoded))
      return
    end
    cb(decoded, nil)
  end,

  base64_encode = function(input, cb)
    local trimmed = string.gsub(input, "^%s+", ""):gsub("%s+$", "")
    local encoded = lc.base64.encode(trimmed)
    cb(encoded, nil)
  end,

  url_encode = function(input, cb)
    local script = "import sys; from urllib.parse import quote; print(quote(sys.stdin.read(), safe=''))"
    lc.system({ "python3", "-c", script }, { stdin = input }, function(out)
      if out.code == 0 then
        cb(out.stdout, nil)
      else
        cb(nil, out.stderr)
      end
    end)
  end,

  url_decode = function(input, cb)
    local script = "import sys; from urllib.parse import unquote; print(unquote(sys.stdin.read()))"
    lc.system({ "python3", "-c", script }, { stdin = input }, function(out)
      if out.code == 0 then
        cb(out.stdout, nil)
      else
        cb(nil, out.stderr)
      end
    end)
  end,

  unix_to_date = function(input, cb)
    input = string.gsub(input, "%s+", "")
    local len = #input

    local unix_time
    if len == 13 then
      unix_time = tonumber(input) / 1000
    elseif len == 10 then
      unix_time = tonumber(input)
    else
      cb(nil, "Invalid Unix timestamp length")
      return
    end

    if not unix_time then
      cb(nil, "Invalid Unix timestamp")
      return
    end

    -- Use lc.time.format to get readable date
    local readable = lc.time.format(unix_time, "%Y-%m-%d %H:%M:%S")

    -- Use lc.time.now to get current time
    local current_time = lc.time.now()
    local diff = unix_time - current_time

    local suffix
    local time_since
    if diff < 0 then
      suffix = " ago"
      time_since = -diff
    else
      suffix = " later"
      time_since = diff
    end

    local secs = time_since
    local mins = math.floor(secs / 60)
    local hours = math.floor(mins / 60)
    local days = math.floor(hours / 24)
    local months = math.floor(days / 30)
    local years = math.floor(days / 365)

    local since_str
    if years > 0 then
      since_str = years .. " year"
    elseif months > 0 then
      since_str = months .. " month"
    elseif days > 0 then
      since_str = days .. " day"
    elseif hours > 0 then
      since_str = hours .. " hour"
    elseif mins > 0 then
      since_str = mins .. " minute"
    else
      since_str = secs .. " second"
    end

    local result_str = readable .. " - " .. since_str .. suffix
    cb(result_str, nil)
  end,

  json_to_nix = function(input, cb)
    local tmp = "/tmp/lazycmd_t_json.json"
    lc.fs.write_file_sync(tmp, input, function(err)
      if err then
        cb(nil, err)
        return
      end

      local expr = 'builtins.fromJSON (builtins.readFile "' .. tmp .. '")'
      lc.system({ "nix", "eval", "--impure", "--expr", expr }, function(out)
        lc.system({ "rm", tmp }, function() end)

        if out.code == 0 then
          cb(out.stdout, nil)
        else
          cb(nil, out.stderr)
        end
      end)
    end)
  end,

  yaml_to_nix = function(input, cb)
    if not has_cmd("yq") then
      cb(nil, "yq not found")
      return
    end

    lc.system({ "yq", "-o=json", "." }, { stdin = input }, function(out)
      if out.code ~= 0 then
        cb(nil, out.stderr)
        return
      end

      tools_impl.json_to_nix(out.stdout, cb)
    end)
  end,

  toml_to_nix = function(input, cb)
    if not has_cmd("yq") then
      cb(nil, "yq not found")
      return
    end

    lc.system({ "yq", "-p", "toml", "-o=json", "." }, { stdin = input }, function(out)
      if out.code ~= 0 then
        cb(nil, out.stderr)
        return
      end

      tools_impl.json_to_nix(out.stdout, cb)
    end)
  end,
}

-- Execute a tool
local function execute_tool(tool_key, cb)
  read_clipboard(function(input, err)
    if err then
      cb(nil, "Failed to read clipboard: " .. err)
      return
    end

    if not input or #input == 0 then
      cb(nil, "Clipboard is empty")
      return
    end

    local func = tools_impl[tool_key]
    if func then
      func(input, cb)
    else
      cb(nil, "Unknown tool: " .. tool_key)
    end
  end)
end

function M.setup()
  -- Check for required commands
  if not has_cmd("python3") then
    lc.log("warn", "t: python3 not found, URL encode/decode will not work")
  end

  if not has_cmd("yq") then
    lc.log("warn", "t: yq not found, YAML/TOML to Nix will not work")
  end

  if not has_cmd("nix") then
    lc.log("warn", "t: nix not found, JSON/YAML/TOML to Nix will not work")
  end

  -- Keymap: r to reload
  lc.keymap.set("main", "r", function()
    lc.cmd("reload")
  end)

  -- Keymap: c to copy result
  lc.keymap.set("main", "c", function()
    local entry = lc.api.page_get_hovered()
    if entry and entry.result then
      write_clipboard(entry.result, function(success, err)
        if success then
          lc.notify("Copied to clipboard")
        else
          lc.notify("Failed to copy: " .. (err or "unknown error"))
        end
      end)
    end
  end)

  lc.keymap.set("main", "<enter>", function()
    local entry = lc.api.page_get_hovered()
    if entry and entry.key then
      execute_tool(entry.key, function(result, err)
        if err then
          lc.api.page_set_preview(lc.style.text {
            lc.style.line {
              lc.style.span("Error: " .. err)
            }
          })
          lc.notify("Error: " .. err)
        else
          -- Store result for c keymap
          entry.result = result

          -- Detect language for highlighting
          local language = nil
          if entry.key == "json_format" or entry.key == "json_minify" or entry.key == "json_to_nix" then
            language = "json"
          elseif entry.key == "yaml_to_nix" then
            language = "yaml"
          elseif entry.key == "toml_to_nix" then
            language = "toml"
          end

          -- Show highlighted result in preview
          if language then
            lc.api.page_set_preview(lc.style.highlight(result, language))
          else
            lc.api.page_set_preview(lc.style.text {
              lc.style.line {
                lc.style.span(result)
              }
            })
          end

          -- Copy to clipboard
          write_clipboard(result, function(success, _)
            if success then
              lc.notify("Result copied to clipboard")
            end
          end)
        end
      end)
    end
  end)
end

function M.list(_, cb)
  local entries = {}
  for _, tool in ipairs(tools) do
    table.insert(entries, {
      key = tool.key,
      display = tool.display,
    })
  end
  cb(entries)
end

function M.preview(entry, cb)
  local descriptions = {
    json_format = "Format JSON with indentation",
    json_minify = "Minify JSON (remove whitespace)",
    stringify = "Convert text to JSON string",
    unstringify = "Convert JSON string to text",
    base64_decode = "Decode Base64 string",
    base64_encode = "Encode string to Base64",
    url_encode = "URL encode string",
    url_decode = "URL decode string",
    unix_to_date = "Convert Unix timestamp to human readable date",
    json_to_nix = "Convert JSON to Nix expression",
    yaml_to_nix = "Convert YAML to Nix expression",
    toml_to_nix = "Convert TOML to Nix expression",
  }

  local desc = descriptions[entry.key] or "No description"

  local preview_text = entry.display .. "\n\n" .. desc .. "\n\n"
    .. "Press Enter to execute (reads from clipboard, writes result back)\n"
    .. "Press c to copy result to clipboard"

  lc.api.page_set_preview(lc.style.text {
    lc.style.line {
      lc.style.span(preview_text)
    }
  })

  cb("")
end

return M
