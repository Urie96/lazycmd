local M = {}

local function parse_container_json(json_str)
  local success, data = pcall(lc.json.decode, json_str)
  if not success or type(data) ~= 'table' then return {} end

  return {
    id = data.ID,
    name = data.Names,
    image = data.Image,
    state = data.State,
    status = data.Status,
    ports = data.Ports or '',
    created = data.CreatedAt,
    command = data.Command,
  }
end

function M.list(cb)
  local cmd = { 'docker', 'ps', '-a', '--format', '{{json .}}' }

  lc.system(cmd, function(out)
    if out.code ~= 0 then
      cb('Failed to list containers', out.stderr or 'Unknown error')
      return
    end

    -- 解析 JSON 输出（每行一个 JSON 对象）
    local entries = {}
    local lines = lc.split(out.stdout, '\n')
    for _, line in ipairs(lines) do
      if line ~= '' then
        local container = parse_container_json(line)
        local state_color = container.state == 'running' and 'green' or 'red'
        local display_parts = {
          container.name:fg(state_color),
          ' ',
          container.image:fg 'blue',
        }

        table.insert(entries, {
          key = container.id,
          display = lc.style.line(display_parts),
          container = container,
        })
      end
    end

    cb(entries)
  end)
end

function M.preview(entry, cb)
  local cmd = { 'docker', 'inspect', entry.container.id }

  -- 显示容器详细信息和日志
  lc.system(cmd, function(inspect_out)
    if inspect_out.code == 0 then
      local success, data = pcall(lc.json.decode, inspect_out.stdout)
      if success and type(data) == 'table' and #data > 0 then
        local info = data[1]
        local container = entry.container
        local lines = {
          'Container: ' .. (container.name or 'unknown'),
          'ID: ' .. (container.id or 'unknown'),
          'Image: ' .. (container.image or ''),
          'State: ' .. (container.state or 'unknown'),
          'Status: ' .. (container.status or ''),
        }
        if container.ports and container.ports ~= '' then table.insert(lines, 'Ports: ' .. container.ports) end
        if container.created then table.insert(lines, 'Created: ' .. container.created) end
        cb(table.concat(lines, '\n'))
        return
      end
    else
      cb('Failed to get container details: ' .. inspect_out.stderr)
    end
  end)
end

return M
