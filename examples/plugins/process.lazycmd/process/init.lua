local M = {}

function M.setup()
  -- 测试 lc.system.executable
  local has_ps = lc.system.executable 'ps'
  local has_pstree = lc.system.executable 'pstree'

  if not has_ps then
    lc.notify 'Error: ps command not found'
    lc.log('error', 'ps command not found')
  elseif not has_pstree then
    lc.log('warn', 'pstree command not found, preview may not work')
  else
    lc.log('info', 'ps and pstree commands are available')
  end

  lc.keymap.set('main', 'ctrl-d', function()
    local entry = lc.api.page_get_hovered()
    if entry and entry.pid then lc.system({ 'kill', tostring(entry.pid) }, function() lc.cmd 'reload' end) end
  end)
end

function M.list(_, cb)
  lc.system({ 'ps', '-eo', 'pid,command' }, function(out)
    local lines = out.stdout:trim():split '\n'
    local entries = {}
    for _, line in ipairs(lines) do
      local pid, cmd = line:match '^%s*(%d+)%s+(.+)$'
      if pid and cmd then
        table.insert(entries, {
          key = pid,
          pid = tonumber(pid),
          display = line,
        })
      end
    end

    cb(entries)
  end)
end

function M.preview(entry, cb)
  lc.system({ 'pstree', '-p', entry.pid }, function(out)
    local preview
    if out.code == 0 then
      preview = out.stdout:ansi()
    else
      preview = out.stderr:ansi()
    end
    cb(preview)
  end)
end

return M
