-- himalaya.lazycmd - Interactive email viewer for himalaya CLI
-- https://github.com/soywod/himalaya

local M = {}

-- 解析账号列表
local function parse_accounts(output)
  local success, data = pcall(lc.json.decode, output.stdout)
  if not success or type(data) ~= 'table' then return {}, (data or 'Invalid JSON') end

  local entries = {}
  for _, account in ipairs(data) do
    table.insert(entries, {
      key = account.name,
      display = account.name,
      account = account.name,
    })
  end

  return entries
end

-- 解析文件夹列表
local function parse_folders(output, account)
  local success, data = pcall(lc.json.decode, output.stdout)
  if not success or type(data) ~= 'table' then return {}, (data or 'Invalid JSON') end

  local entries = {}
  for _, folder in ipairs(data) do
    local display = folder.name
    if folder.unseen and folder.unseen > 0 then display = display .. ' (' .. tostring(folder.unseen) .. ')' end
    table.insert(entries, {
      key = folder.name,
      display = display,
      account = account,
    })
  end

  return entries
end

-- 解析信封列表
local function parse_envelopes(output, account, folder)
  local success, data = pcall(lc.json.decode, output.stdout)
  if not success or type(data) ~= 'table' then return {}, (data or 'Invalid JSON') end

  local entries = {}
  for _, envelope in ipairs(data) do
    local display = envelope.subject or '(no subject)'

    -- 添加发件人信息
    if envelope.from and envelope.from.name then
      display = display .. ' - ' .. envelope.from.name
    elseif envelope.from and envelope.from.address then
      display = display .. ' - ' .. envelope.from.address
    end

    -- 添加附件标记
    if envelope.has_attachments then display = display .. ' [A]' end

    table.insert(entries, {
      key = tostring(envelope.id),
      display = display,
      id = envelope.id,
      account = account,
      folder = folder,
    })
  end

  return entries
end

-- 解析邮件内容
local function parse_message(output)
  local success, data = pcall(lc.json.decode, output.stdout)
  if not success or type(data) ~= 'table' then return nil, (data or 'Invalid JSON') end
  return data
end

-- 构建邮件预览
local function build_preview(message)
  if not message then return 'No message data' end

  local lines = {}

  -- 主题
  if message.subject then
    table.insert(lines, 'Subject: ' .. message.subject)
    table.insert(lines, '')
  end

  -- 发件人
  if message.from then
    local from_str = message.from.name and (message.from.name .. ' <' .. message.from.address .. '>')
      or message.from.address
      or 'Unknown'
    table.insert(lines, 'From: ' .. from_str)
  end

  -- 收件人
  if message.to and #message.to > 0 then
    local to_str = {}
    for _, to in ipairs(message.to) do
      local addr = to.name and (to.name .. ' <' .. to.address .. '>') or to.address
      table.insert(to_str, addr)
    end
    table.insert(lines, 'To: ' .. table.concat(to_str, ', '))
  end

  -- 抄送
  if message.cc and #message.cc > 0 then
    local cc_str = {}
    for _, cc in ipairs(message.cc) do
      local addr = cc.name and (cc.name .. ' <' .. cc.address .. '>') or cc.address
      table.insert(cc_str, addr)
    end
    table.insert(lines, 'Cc: ' .. table.concat(cc_str, ', '))
  end

  -- 日期
  if message.date then table.insert(lines, 'Date: ' .. message.date) end

  -- 附件
  if message.attachments and #message.attachments > 0 then
    table.insert(lines, '')
    table.insert(lines, 'Attachments: ' .. tostring(#message.attachments))
    for _, att in ipairs(message.attachments) do
      local att_name = att.name or '(no name)'
      local att_size = att.size and (' (' .. tostring(att.size) .. ' bytes)') or ''
      table.insert(lines, '  - ' .. att_name .. att_size)
    end
  else
    table.insert(lines, '')
    table.insert(lines, 'Attachments: none')
  end

  -- 分隔线
  table.insert(lines, '')
  table.insert(lines, string.rep('─', 50))
  table.insert(lines, '')

  -- 邮件正文
  if message.body then
    -- 如果 body 是文本
    if type(message.body) == 'string' then
      -- 分割成行
      for line in string.gmatch(message.body, '[^\n]+') do
        table.insert(lines, line)
      end
    elseif type(message.body) == 'table' and message.body.text then
      -- 如果 body 有 text 字段
      for line in string.gmatch(message.body.text, '[^\n]+') do
        table.insert(lines, line)
      end
    end
  end

  return table.concat(lines, '\n')
end

-- 列出内容
function M.list(path, cb)
  -- 根路径：列出所有账号
  if #path == 0 then
    lc.system({ 'himalaya', '--output', 'json', 'account', 'list' }, function(output)
      if output.code ~= 0 then
        lc.log('error', 'Failed to list accounts: {}', output.stderr or 'Unknown error')
        cb {}
        return
      end

      local entries, err = parse_accounts(output)
      if err then
        lc.log('error', 'Failed to parse accounts: {}', err)
        cb {}
        return
      end

      cb(entries)
    end)

  -- 账号路径：列出该账号的文件夹
  elseif #path == 1 then
    local account = path[1]
    lc.system({ 'himalaya', '--output', 'json', 'folder', 'list', '--account', account }, function(output)
      if output.code ~= 0 then
        lc.log('error', 'Failed to list folders: {}', output.stderr or 'Unknown error')
        cb {}
        return
      end

      local entries, err = parse_folders(output, account)
      if err then
        lc.log('error', 'Failed to parse folders: {}', err)
        cb {}
        return
      end

      cb(entries)
    end)

  -- 文件夹路径：列出该文件夹的信封
  elseif #path == 2 then
    local account = path[1]
    local folder = path[2]
    lc.system({
      'himalaya',
      '--output',
      'json',
      'envelope',
      'list',
      '--account',
      account,
      '--folder',
      folder,
    }, function(output)
      if output.code ~= 0 then
        lc.log('error', 'Failed to list envelopes: {}', output.stderr or 'Unknown error')
        cb {}
        return
      end

      local entries, err = parse_envelopes(output, account, folder)
      if err then
        lc.log('error', 'Failed to parse envelopes: {}', err)
        cb {}
        return
      end

      cb(entries)
    end)

  -- 其他路径：返回空
  else
    cb {}
  end
end

-- 预览内容
function M.preview(entry, cb)
  if not entry or not entry.id or not entry.account or not entry.folder then
    cb 'Select an email to preview'
    return
  end

  lc.system({
    'himalaya',
    '--output',
    'json',
    'message',
    'read',
    tostring(entry.id),
    '--account',
    entry.account,
    '--folder',
    entry.folder,
  }, function(output)
    if output.code ~= 0 then
      local error_msg = output.stderr or 'Unknown error'
      lc.log('error', 'Failed to read message: {}', error_msg)
      cb('Error: ' .. error_msg)
      return
    end

    local message, err = parse_message(output)
    if err then
      lc.log('error', 'Failed to parse message: {}', err)
      cb('Error: ' .. err)
      return
    end

    local preview = build_preview(message)
    cb(preview)
  end)
end

-- 设置插件
function M.setup()
  -- 右键：进入下一级
  lc.keymap.set('main', '<right>', function()
    local path = lc.api.get_current_path()
    local entry = lc.api.page_get_hovered()

    if not entry then
      lc.notify 'No entry selected'
      return
    end

    -- 在信封层级，不能再进入下一级
    if #path >= 2 then
      lc.notify 'Cannot enter further'
      return
    end

    -- 构建下一级路径
    local new_path = {}
    for _, p in ipairs(path) do
      table.insert(new_path, p)
    end
    table.insert(new_path, entry.key)

    lc.api.go_to(new_path)
  end)

  -- 左键：返回上一级
  lc.keymap.set('main', '<left>', function()
    local path = lc.api.get_current_path()
    if #path == 0 then
      lc.notify 'Already at root'
      return
    end

    -- 移除最后一级
    local new_path = {}
    for i = 1, #path - 1 do
      table.insert(new_path, path[i])
    end

    lc.api.go_to(new_path)
  end)
end

return M
