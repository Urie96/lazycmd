local M = {}

local config = {
  token = '',
  base_url = '',
  editor = os.getenv 'EDITOR' or 'vim',
  visibility = 'PRIVATE',
}

local function api_call(method, path, body, cb)
  lc.log('debug', 'API call: {} {}', method, path)

  lc.http.request({
    method = method,
    url = config.base_url .. '/api/v1' .. path,
    headers = {
      ['Authorization'] = 'Bearer ' .. config.token,
      ['Content-Type'] = 'application/json',
    },
    body = body and lc.json.encode(body),
  }, function(response) cb(response) end)
end

-- Create a temporary file for editing
local function create_temp_file(prefix, suffix)
  local timestamp = os.time()
  local tmp_path = '/tmp/' .. prefix .. '_' .. timestamp .. suffix
  return tmp_path
end

-- Edit the currently selected memo
local function edit_current_memo()
  local entry = lc.api.page_get_hovered()

  if not entry or not entry.memo then
    lc.notify 'No memo selected'
    lc.log('warn', 'No memo selected for editing')
    return
  end

  local memo = entry.memo

  lc.log('info', 'Editing memo #{}', memo.id)

  -- Create temporary file with current content
  local temp_file = create_temp_file('memo', '.md')
  local content = memo.content or ''

  -- Write current content to temp file
  local success, err = lc.fs.write_file_sync(temp_file, content)
  if not success then
    lc.notify('Failed to create temp file: ' .. (err or 'Unknown error'))
    return
  end

  -- Open editor with the temp file
  lc.interactive({ config.editor, temp_file }, function(exit_code)
    lc.log('debug', 'Editor exited with code: {}', exit_code)

    -- Read edited content
    local new_content, read_err = lc.fs.read_file_sync(temp_file)
    if not new_content then
      lc.log('error', 'Failed to read temp file: {}', read_err or 'Unknown')
      lc.notify 'Error: Failed to read edited content'
      return
    end

    -- Check if content was changed
    if new_content == content then
      lc.notify 'No changes made'
      return
    end

    -- Update memo via API
    lc.log('info', 'Updating memo #{} to {}', memo.id, new_content)

    api_call('PATCH', '/memos/' .. memo.id, {
      content = new_content,
    }, function(res)
      if not res.success then
        lc.notify('Failed to update memo: ' .. (res.error or 'Unknown error'))
        lc.log('error', 'Failed to update memo: {}', res.error or 'Unknown')
        return
      end

      lc.notify 'Memo updated successfully'

      -- Reload the list to show updated content
      lc.cmd 'reload'
    end)

    -- Clean up temp file
    os.remove(temp_file)
  end)
end

-- Copy the content of currently selected memo to clipboard
local function yank_current_memo()
  local entry = lc.api.page_get_hovered()

  if not entry or not entry.memo or not entry.memo.content then
    lc.notify 'No memo selected'
    lc.log('warn', 'No memo selected for yanking')
    return
  end

  local memo = entry.memo

  lc.log('info', 'Yanking memo #{} content', memo.id)

  -- Copy content to clipboard using OSC 52
  local success, err = pcall(lc.osc52_copy, lc.trim(memo.content))

  if not success then
    lc.notify('Failed to copy: ' .. tostring(err))
  else
    lc.notify 'Copied to clipboard'
  end
end

-- Delete the currently selected memo
local function delete_current_memo()
  local entry = lc.api.page_get_hovered()

  if not entry then
    lc.notify 'No memo selected'
    lc.log('warn', 'No memo selected for deletion')
    return
  end

  local memo = entry.memo

  api_call('DELETE', '/memos/' .. memo.id, nil, function(res)
    if not res.success then
      lc.notify('Failed to delete memo: ' .. (res.error or 'Unknown error'))
      lc.log('error', 'Failed to delete memo: {}', res.error or 'Unknown')
      return
    end

    lc.notify('Memo deleted successfully: ' .. memo.id)
    lc.cmd 'reload'
  end)
end

-- Create a new memo using external editor
local function create_new_memo()
  local temp_file = create_temp_file('new_memo', '.md')
  local template = ''
  lc.fs.write_file_sync(temp_file, template)

  -- Open editor
  lc.interactive({ config.editor, temp_file }, function(exit_code)
    lc.log('debug', 'Editor exited with code: {}', exit_code)

    -- Read edited content
    local content, err = lc.fs.read_file_sync(temp_file)
    os.remove(temp_file)
    if err then
      lc.notify('Error: Failed to read edited content' .. err)
      return
    end

    if not content then
      lc.notify 'Failed to read edited content'
      return
    end

    if content:match '^%s*$' then
      lc.notify 'No content provided'
      return
    end

    api_call('POST', '/memos', {
      content = content,
      visibility = config.visibility,
    }, function(res)
      if not res.success then
        lc.notify('Failed to create memo: ' .. (res.error or 'Unknown error'))
        return
      end

      local result = lc.json.decode(res.body)
      if result and result.name then
        lc.notify('Memo created: ' .. result.name)
      else
        lc.notify 'Memo created successfully'
      end

      lc.cmd 'reload'
      lc.cmd 'scroll_by -9999' -- 返回顶部
    end)
  end)
end

function M.setup(opt)
  config = lc.tbl_extend(config, opt or {})
  lc.keymap.set('main', 'n', create_new_memo)
  lc.keymap.set('main', 'y', yank_current_memo)
  lc.keymap.set('main', '<C-e>', edit_current_memo)
  lc.keymap.set('main', '<enter>', edit_current_memo)
  lc.keymap.set('main', 'dd', delete_current_memo)
end

function M.list(_, cb)
  lc.log('info', 'Loading memos list')
  lc.api.page_set_preview 'Loading memos...'

  api_call('GET', '/memos?state=NORMAL&pageSize=100', nil, function(res)
    if not res.success then
      lc.notify('Error: ' .. (res.error or 'Unknown error'))
      return
    end

    -- Parse JSON response
    local memos = lc.json.decode(res.body)

    -- Handle error response from memos API
    if type(memos) ~= 'table' or #memos.memos == 0 then
      lc.notify 'No memos found'
      cb {}
      return
    end

    -- Convert memos to PageEntry format
    local entries = {}
    for _, memo in ipairs(memos.memos) do
      local content = memo.content or ''
      memo.id = memo.name:sub(7)
      local display_title = content:sub(1, 60)

      -- Truncate and add ellipsis if content is too long
      if #content > 60 then display_title = display_title .. '...' end

      table.insert(entries, {
        key = tostring(memo.id),
        display = display_title,
        memo = memo, -- Store full memo data for preview
      })
    end

    lc.log('info', 'Loaded {} memos entries', #entries)
    cb(entries)
  end)
end

-- Format timestamp to readable date
local function format_timestamp(ts)
  if not ts then return 'Unknown' end

  -- Use lc.time.parse to parse ISO 8601 format
  local success, timestamp = pcall(lc.time.parse, ts)
  if success and timestamp then
    -- Format timestamp as local time
    return os.date('%Y-%m-%d %H:%M:%S', timestamp)
  end

  -- Fallback to original string if parsing fails
  return ts
end

-- Build rich preview text for a memo
local function build_preview(memo)
  local lines = {}

  -- Meta info
  table.insert(lines, '📝 Metadata:')
  table.insert(lines, '   ID:           ' .. memo.id)
  table.insert(lines, '   State:        ' .. (memo.state or 'UNKNOWN'))
  table.insert(lines, '   Visibility:   ' .. (memo.visibility or 'PRIVATE'))
  table.insert(lines, '   Created:      ' .. format_timestamp(memo.createTime))
  table.insert(lines, '   Updated:      ' .. format_timestamp(memo.updateTime))

  -- Pinned status
  if memo.pinned then table.insert(lines, '   📌 Pinned') end

  -- Tags
  if memo.tags and #memo.tags > 0 then
    local tag_list = {}
    for _, tag in ipairs(memo.tags) do
      table.insert(tag_list, '#' .. tag)
    end
    table.insert(lines, '   Tags:         ' .. table.concat(tag_list, ' '))
  else
    table.insert(lines, '   Tags:         (none)')
  end

  -- Attachments
  if memo.attachments and #memo.attachments > 0 then
    table.insert(lines, '   📎 Attachments: ' .. #memo.attachments)
  end

  -- Relations
  if memo.relations and #memo.relations > 0 then table.insert(lines, '   🔗 Relations:   ' .. #memo.relations) end

  -- Reactions
  if memo.reactions and #memo.reactions > 0 then table.insert(lines, '   ❤️  Reactions:  ' .. #memo.reactions) end

  -- Properties
  table.insert(lines, '')
  table.insert(lines, '⚙️ Properties:')
  if memo.property then
    local props = {}
    if memo.property.hasLink then table.insert(props, 'links') end
    if memo.property.hasTaskList then table.insert(props, 'tasks') end
    if memo.property.hasCode then table.insert(props, 'code') end
    if memo.property.hasIncompleteTasks then table.insert(props, 'incomplete_tasks') end
    if #props > 0 then
      table.insert(lines, '   ' .. table.concat(props, ', '))
    else
      table.insert(lines, '   (none)')
    end
  end

  -- Content
  table.insert(lines, '')
  table.insert(lines, '📄 Content:')
  table.insert(lines, '')
  table.insert(lines, memo.content or '(no content)')

  return table.concat(lines, '\n')
end

function M.preview(entry, cb) cb(build_preview(entry.memo)) end

return M
