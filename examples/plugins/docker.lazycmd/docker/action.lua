local M = {}

-- 辅助函数：获取当前选中的资源信息
local function get_selected_entry()
  local entry = lc.api.page_get_hovered()
  return entry
end

-- 执行 Docker 操作
local function do_docker_action(action_name, args, interactive)
  local entry = get_selected_entry()
  if not entry then
    lc.notify 'Please select a resource first'
    return
  end

  local cmd = { 'docker', action_name }
  if args then
    for _, arg in ipairs(args) do
      table.insert(cmd, arg)
    end
  end
  table.insert(cmd, entry.resource_type == 'container' and entry.id or entry.name)

  if interactive then
    lc.interactive(cmd, { wait_confirm = function(exit_code) return exit_code ~= 0 end }, function(exit_code)
      if exit_code == 0 then
        lc.notify(action_name .. ' for ' .. entry.name .. ' successfully')
        lc.cmd 'reload'
      else
        lc.notify(action_name .. ' for ' .. entry.name .. ' failed')
      end
    end)
  else
    lc.system(cmd, function(out)
      if out.code == 0 then
        lc.notify(action_name .. ' for ' .. entry.name .. ' successfully')
        lc.cmd 'reload'
      else
        lc.notify(action_name .. ' for ' .. entry.name .. ' failed')
      end
    end)
  end
end

function M.logs()
  local entry = get_selected_entry()
  if not entry or entry.resource_type ~= 'container' then
    lc.notify 'Please select a container first'
    return
  end
  lc.interactive { 'docker', 'logs', '--follow', entry.id }
end

function M.inspect()
  local entry = get_selected_entry()
  if not entry then
    lc.notify 'Please select a resource first'
    return
  end
  lc.interactive { 'docker', 'inspect', entry.resource_type == 'container' and entry.id or entry.name }
end

function M.exec()
  local entry = get_selected_entry()
  if not entry or entry.resource_type ~= 'container' then
    lc.notify 'Please select a container first'
    return
  end

  lc.interactive { 'docker', 'exec', '-it', entry.id, '/bin/sh' }
end

function M.stats()
  local entry = get_selected_entry()
  if not entry or entry.resource_type ~= 'container' then
    lc.notify 'Please select a container first'
    return
  end
  lc.interactive { 'docker', 'stats', '--no-stream', entry.id }
end

-- 镜像操作
function M.remove_image()
  local entry = get_selected_entry()
  if not entry or entry.resource_type ~= 'image' then
    lc.notify 'Please select an image first'
    return
  end

  lc.confirm('Remove image ' .. entry.name .. '?', function(confirmed)
    if not confirmed then return end

    lc.system({ 'docker', 'rmi', entry.id }, function(out)
      if out.code == 0 then
        lc.notify('Image ' .. entry.name .. ' removed successfully')
        lc.cmd 'reload'
      else
        lc.notify('Failed to remove image ' .. entry.name .. ': ' .. (out.stderr or 'Unknown error'))
      end
    end)
  end)
end

function M.pull()
  local entry = get_selected_entry()
  if not entry or entry.resource_type ~= 'image' then
    lc.notify 'Please select an image first'
    return
  end
  lc.interactive({ 'docker', 'pull', entry.name }, { wait_confirm = function() return false end }, function(exit_code)
    if exit_code == 0 then
      lc.notify('Image ' .. entry.name .. ' pulled successfully')
      lc.cmd 'reload'
    else
      lc.notify('Failed to pull image ' .. entry.name)
    end
  end)
end

function M.build() lc.interactive { 'docker', 'build', '-t', 'myapp', '.' } end

-- 卷操作
function M.remove_volume()
  local entry = get_selected_entry()
  if not entry or entry.resource_type ~= 'volume' then
    lc.notify 'Please select a volume first'
    return
  end

  lc.confirm('Remove volume ' .. entry.name .. '?', function(confirmed)
    if not confirmed then return end

    lc.system({ 'docker', 'volume', 'rm', entry.name }, function(out)
      if out.code == 0 then
        lc.notify('Volume ' .. entry.name .. ' removed successfully')
        lc.cmd 'reload'
      else
        lc.notify('Failed to remove volume ' .. entry.name .. ': ' .. (out.stderr or 'Unknown error'))
      end
    end)
  end)
end

function M.create_volume()
  lc.interactive({ 'docker', 'volume', 'create' }, { wait_confirm = function() return false end }, function(exit_code)
    if exit_code == 0 then
      lc.notify 'Volume created successfully'
      lc.cmd 'reload'
    else
      lc.notify 'Failed to create volume'
    end
  end)
end

-- 网络操作
function M.remove_network()
  local entry = get_selected_entry()
  if not entry or entry.resource_type ~= 'network' then
    lc.notify 'Please select a network first'
    return
  end

  lc.confirm('Remove network ' .. entry.name .. '?', function(confirmed)
    if not confirmed then return end

    lc.system({ 'docker', 'network', 'rm', entry.id }, function(out)
      if out.code == 0 then
        lc.notify('Network ' .. entry.name .. ' removed successfully')
        lc.cmd 'reload'
      else
        lc.notify('Failed to remove network ' .. entry.name .. ': ' .. (out.stderr or 'Unknown error'))
      end
    end)
  end)
end

function M.create_network()
  lc.interactive({ 'docker', 'network', 'create' }, { wait_confirm = function() return false end }, function(exit_code)
    if exit_code == 0 then
      lc.notify 'Network created successfully'
      lc.cmd 'reload'
    else
      lc.notify 'Failed to create network'
    end
  end)
end

function M.connect()
  local entry = get_selected_entry()
  if not entry or entry.resource_type ~= 'network' then
    lc.notify 'Please select a network first'
    return
  end

  lc.interactive { 'docker', 'network', 'connect', entry.id }
end

function M.disconnect()
  local entry = get_selected_entry()
  if not entry or entry.resource_type ~= 'network' then
    lc.notify 'Please select a network first'
    return
  end

  lc.interactive { 'docker', 'network', 'disconnect', entry.id }
end

-- 选择操作
function M.select_action()
  local entry = get_selected_entry()
  if not entry then
    lc.notify 'Please select a resource first'
    return
  end

  local options = {}

  -- 添加 inspect 选项（所有资源）
  table.insert(options, {
    value = 'inspect',
    display = lc.style.line { ('🔍 Inspect'):fg 'cyan' },
  })

  if entry.resource_type == 'container' then
    -- 容器操作
    if entry.state == 'running' then
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
    elseif entry.state == 'paused' then
      table.insert(options, {
        value = 'unpause',
        display = lc.style.line { ('▶️ Unpause'):fg 'green' },
      })
      table.insert(options, {
        value = 'stop',
        display = lc.style.line { ('⏹️ Stop'):fg 'red' },
      })
    else
      table.insert(options, {
        value = 'start',
        display = lc.style.line { ('▶️ Start'):fg 'green' },
      })
    end

    table.insert(options, {
      value = 'logs',
      display = lc.style.line { ('📋 Logs'):fg 'blue' },
    })
    table.insert(options, {
      value = 'exec',
      display = lc.style.line { ('💻 Exec'):fg 'yellow' },
    })
    table.insert(options, {
      value = 'stats',
      display = lc.style.line { ('📊 Stats'):fg 'magenta' },
    })
    table.insert(options, {
      value = 'remove_container',
      display = lc.style.line { ('🗑️ Remove'):fg 'red' },
    })
  elseif entry.resource_type == 'image' then
    -- 镜像操作
    table.insert(options, {
      value = 'pull',
      display = lc.style.line { ('⬇️ Pull'):fg 'green' },
    })
    table.insert(options, {
      value = 'remove_image',
      display = lc.style.line { ('🗑️ Remove'):fg 'red' },
    })
  elseif entry.resource_type == 'volume' then
    -- 卷操作
    table.insert(options, {
      value = 'remove_volume',
      display = lc.style.line { ('🗑️ Remove'):fg 'red' },
    })
  elseif entry.resource_type == 'network' then
    -- 网络操作
    table.insert(options, {
      value = 'connect',
      display = lc.style.line { ('🔗 Connect'):fg 'green' },
    })
    table.insert(options, {
      value = 'disconnect',
      display = lc.style.line { ('🔌 Disconnect'):fg 'yellow' },
    })
    table.insert(options, {
      value = 'remove_network',
      display = lc.style.line { ('🗑️ Remove'):fg 'red' },
    })
  end

  lc.select({
    prompt = 'Select an action',
    options = options,
  }, function(choice)
    if choice then M[choice]() end
  end)
end

return M
