-- Date and time related tools

-- Read from clipboard
local function read_clipboard()
  local ok, content = pcall(lc.clipboard.get)
  if ok then
    return content
  else
    return nil, content
  end
end

-- Write to clipboard
local function write_clipboard(text)
  local ok, err = pcall(lc.clipboard.set, text)
  return ok, err
end

-- Show result in preview
local function show_preview(result)
  lc.api.page_set_preview(lc.style.text {
    lc.style.line {
      lc.style.span(result),
    },
  })
end

-- Show error in preview
local function show_error(err)
  lc.api.page_set_preview(lc.style.text {
    lc.style.line {
      lc.style.span('Error: ' .. err),
    },
  })
end

local function unix_to_date(cb)
  local input, err = read_clipboard()
  if err or not input or #input == 0 then
    show_error(err or 'Clipboard is empty')
    return
  end

  input = string.gsub(input, '%s+', '')
  local len = #input

  local unix_time
  if len == 13 then
    unix_time = tonumber(input) / 1000
  elseif len == 10 then
    unix_time = tonumber(input)
  else
    show_error 'Invalid Unix timestamp length'
    return
  end

  if not unix_time then
    show_error 'Invalid Unix timestamp'
    return
  end

  -- Use lc.time.format to get readable date
  local readable = lc.time.format(unix_time, '%Y-%m-%d %H:%M:%S')

  -- Use lc.time.now to get current time
  local current_time = lc.time.now()
  local diff = unix_time - current_time

  local suffix
  local time_since
  if diff < 0 then
    suffix = ' ago'
    time_since = -diff
  else
    suffix = ' later'
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
    since_str = years .. ' year'
  elseif months > 0 then
    since_str = months .. ' month'
  elseif days > 0 then
    since_str = days .. ' day'
  elseif hours > 0 then
    since_str = hours .. ' hour'
  elseif mins > 0 then
    since_str = mins .. ' minute'
  else
    since_str = secs .. ' second'
  end

  local result_str = readable .. ' - ' .. since_str .. suffix
  cb(result_str)
end

return {
  {
    key = 'unix_to_date',
    display = 'Unix Timestamp To Date',
    description = 'Convert Unix timestamp to human readable date',
    on_enter = function() unix_to_date(show_preview) end,
    on_copy = function() unix_to_date(write_clipboard) end,
  },
}
