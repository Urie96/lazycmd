local M = {}

function M.setup() end

function M.list(_, cb)
  lc.system({ 'ps', '-eo', 'pid,command' }, function(out)
    local lines = lc.split(out.stdout, '\n')
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

function M.preview(path, cb)
  local pid = path[#path]
  lc.system({ 'pstree', '-p', pid }, function(out)
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
