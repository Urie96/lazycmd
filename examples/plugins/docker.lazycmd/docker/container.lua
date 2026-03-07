local promise = require 'promise'

local M = {}

local function exec(cmd)
  return promise.new(function(resolve, reject)
    lc.system(cmd, function(output)
      if output.code == 0 then
        resolve(output.stdout)
      else
        reject(output.stderr)
      end
    end)
  end)
end

local function docker_container_list()
  local cmd = { 'docker', 'ps', '-a', '--format', '{{json .}}' }

  return exec(cmd):next(function(stdout)
    local containers = lc.tbl_map(function(line)
      local success, data = pcall(lc.json.decode, line)
      assert(success and type(data) == 'table', 'Failed to parse JSON output: ' .. line)

      return {
        id = data.ID,
        name = data.Names,
        image = data.Image,
        state = data.State,
        status = data.Status,
      }
    end, stdout:trim():split '\n')

    return containers
  end)
end

local function docker_inspect_container(container_id)
  local cmd = { 'docker', 'inspect', container_id }

  return exec(cmd):next(function(stdout)
    local success, data = pcall(lc.json.decode, stdout)
    assert(success and type(data) == 'table' and #data > 0, 'Failed to parse JSON output: ' .. stdout)
    return data[1]
  end)
end

function M.list(cb)
  docker_container_list()
    :next(function(containers)
      local state_info = {
        running = { priority = 1, color = 'green' },
        exited = { priority = 2, color = 'red' },
        created = { priority = 3, color = 'yellow' },
      }

      table.sort(containers, function(a, b)
        local a_priority = state_info[a.state] and state_info[a.state].priority or 4
        local b_priority = state_info[b.state] and state_info[b.state].priority or 4

        return a_priority < b_priority
      end)

      local entries = lc.tbl_map(function(container)
        local display_parts = {
          container.name:fg(state_info[container.state] and state_info[container.state].color or 'white'),
          ' ',
          container.image:fg 'blue',
        }

        return {
          key = container.id,
          display = lc.style.line(display_parts),
          container = container,
        }
      end, containers)

      local lines = lc.tbl_map(function(line) return line.display end, entries)

      lc.style.align_columns(lines)

      cb(entries)
    end)
    :catch(function(err) lc.notify('Failed to list containers: ' .. err) end)
end

function M.preview(entry, cb)
  local detail_area = docker_inspect_container(entry.container.id)
    :next(function(detail)
      local container = entry.container
      local config = detail.Config or {}
      local lines = {
        'ID: ' .. (container.id or 'unknown'),
        'State: ' .. (container.state or 'unknown'),
        'Status: ' .. (container.status or 'unknown'),
        'Command: ' .. table.concat(config.Cmd or {}, ' '),
        'Entrypoint: ' .. table.concat(config.Entrypoint or {}, ''),
      }
      if container.ports and container.ports ~= '' then table.insert(lines, 'Ports: ' .. container.ports) end
      if container.created then table.insert(lines, 'Created: ' .. container.created) end
      return lines
    end)
    :catch(function(err) lc.notify('Failed to get container details: ' .. err) end)

  local log_area = exec({ 'docker', 'logs', entry.container.id, '--tail', '35' })
    :next(function(logs)
      local lines = {
        'Logs: ',
        logs,
      }
      return lines
    end)
    :catch(function(err) lc.notify('Failed to get container logs: ' .. err) end)

  promise.all({ detail_area, log_area }):next(function(results)
    local lines = results[1]
    table.insert(lines, ' ')
    lc.list_extend(lines, results[2])
    cb(lc.style.text(lines))
  end)
end

-- 辅助函数：获取当前选中的容器信息
local function get_selected_container()
  local entry = lc.api.page_get_hovered()
  if not entry or not entry.container then return nil end
  return entry
end

function M.select_action()
  local entry = get_selected_container()
  if not entry then
    lc.notify 'Please select a container first'
    return
  end

  local container = entry.container
  local options = {}

  -- 根据容器状态显示不同操作
  if container.state == 'running' then
    table.insert(options, {
      value = 'follow_logs',
      display = lc.style.line { ('📋 Logs'):fg 'blue' },
    })
    table.insert(options, {
      value = 'exec',
      display = lc.style.line { ('💻 Exec'):fg 'yellow' },
    })
    table.insert(options, {
      value = 'stop',
      display = lc.style.line { ('⏹️ Stop'):fg 'red' },
    })
    table.insert(options, {
      value = 'restart',
      display = lc.style.line { ('🔄 Restart'):fg 'blue' },
    })
    table.insert(options, {
      value = 'pause',
      display = lc.style.line { ('⏸️ Pause'):fg 'yellow' },
    })
  elseif container.state == 'paused' then
    table.insert(options, {
      value = 'unpause',
      display = lc.style.line { ('▶️ Unpause'):fg 'green' },
    })
    table.insert(options, {
      value = 'stop',
      display = lc.style.line { ('⏹️ Stop'):fg 'red' },
    })
    table.insert(options, {
      value = 'stats',
      display = lc.style.line { ('📊 Stats'):fg 'magenta' },
    })
  else
    -- exited, created 等状态
    table.insert(options, {
      value = 'start',
      display = lc.style.line { ('▶️ Start'):fg 'green' },
    })

    table.insert(options, {
      value = 'remove',
      display = lc.style.line { ('🗑️ Remove'):fg 'red' },
    })
  end

  table.insert(options, {
    value = 'inspect',
    display = lc.style.line { ('🔍 Inspect'):fg 'cyan' },
  })

  lc.select({
    prompt = 'Select an action',
    options = options,
  }, function(choice)
    if choice then M[choice](container) end
  end)
end

local function operate_container(action_name, container_name)
  lc.notify(string.format('Operating container %s: %s', container_name, action_name))
  lc.system({ 'docker', action_name, container_name }, function(out)
    if out.code == 0 then
      lc.notify(string.format('Container %s %sed successfully', container_name, action_name))
      lc.cmd 'reload'
    else
      lc.notify(string.format('Failed to %s container %s', action_name, container_name))
    end
  end)
end

-- 容器操作
function M.start(container) operate_container('start', container.name) end

function M.stop(container) operate_container('stop', container.name) end

function M.restart(container) operate_container('restart', container.name) end

function M.pause(container) operate_container('pause', container.name) end

function M.unpause(container) operate_container('unpause', container.name) end

function M.remove(container) operate_container('rm', container.name) end

function M.follow_logs(container) lc.interactive { 'docker', 'logs', '--follow', container.id } end

function M.exec(container) lc.interactive { 'docker', 'exec', '-it', container.id, '/bin/sh' } end

function M.stats(container) lc.interactive { 'docker', 'stats', container.id } end

function M.inspect()
  local entry = get_selected_container()
  if not entry or not entry.container then
    lc.notify 'Please select a container first'
    return
  end
  exec({ 'docker', 'inspect', entry.container.id })
    :next(function(stdout)
      local highlight_inspect = lc.style.highlight(stdout, 'json')
      lc.log('info', 'highlight: {:#?}', highlight_inspect)
      lc.api.page_set_preview(highlight_inspect)
    end)
    :catch(function(stderr) end)
end

function M.remove_container()
  local entry = get_selected_container()
  if not entry then
    lc.notify 'Please select a container first'
    return
  end

  lc.confirm {
    prompt = 'Remove container ' .. entry.container.name .. '?',
    on_confirm = function()
      lc.interactive(
        { 'docker', 'rm', entry.container.id },
        { wait_confirm = function() return false end },
        function(exit_code)
          if exit_code == 0 then
            lc.notify('Container ' .. entry.container.name .. ' removed successfully')
            lc.cmd 'reload'
          else
            lc.notify('Failed to remove container ' .. entry.container.name)
          end
        end
      )
    end,
  }
end

return M
