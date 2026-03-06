-- docker.lazycmd - Docker manager

local M = {}

function M.setup()
  lc.keymap.set('main', '<enter>', function()
    local path = lc.api.get_current_path()
    if #path < 2 then
      lc.cmd 'enter'
    else
      require('docker.action').select_action()
    end
  end)
end

-- 第1级：显示资源类型
local function list_level_1(cb)
  cb {
    { key = 'container', display = ('📦 Containers'):fg 'yellow' },
    { key = 'image', display = ('🖼️ Images'):fg 'yellow' },
    { key = 'volume', display = ('💾 Volumes'):fg 'yellow' },
    { key = 'network', display = ('🌐 Networks'):fg 'yellow' },
  }
end

-- 第2级：显示指定类型的所有资源
local function list_level_2(path, cb)
  local resource_type = path[1]

  if resource_type == 'container' then require('docker.container').list(cb) end
end

function M.list(path, cb)
  if #path == 0 then
    -- 第1级：资源类型
    list_level_1(cb)
  elseif #path == 1 then
    -- 第2级：具体资源
    list_level_2(path, cb)
  else
    cb {}
  end
end

function M.preview(entry, cb)
  local path = lc.api.get_current_path()

  -- 第1级：显示提示信息
  if #path == 0 then
    cb 'Select a resource type to view'
    return
  elseif #path == 1 then
    if path[1] == 'container' then require('docker.container').preview(entry, cb) end
  else
  end

  -- 第2级：显示资源详细状态
  -- if #path == 1 and entry and entry.resource_type then
  --   local resource_type = path[1]
  --   local cmd = {}
  --
  --   if resource_type == 'container' then
  --     -- 显示容器详细信息和日志
  --     cmd = { 'docker', 'inspect', entry.id }
  --     lc.system(cmd, function(inspect_out)
  --       if inspect_out.code == 0 then
  --         local success, data = pcall(lc.json.decode, inspect_out.stdout)
  --         if success and type(data) == 'table' and #data > 0 then
  --           local lines = {
  --             'Container: ' .. (entry.name or 'unknown'),
  --             '',
  --             'ID: ' .. (entry.id or 'unknown'),
  --             'Image: ' .. (entry.image or ''),
  --             'State: ' .. (entry.state or 'unknown'),
  --             'Status: ' .. (entry.status or ''),
  --           }
  --           if entry.ports and entry.ports ~= '' then table.insert(lines, 'Ports: ' .. entry.ports) end
  --           if entry.created then table.insert(lines, 'Created: ' .. entry.created) end
  --           cb(table.concat(lines, '\n'))
  --           return
  --         end
  --       end
  --       cb 'Failed to get container details'
  --     end)
  --     return
  --   elseif resource_type == 'image' then
  --     -- 显示镜像详细信息
  --     -- 使用镜像名称而不是 ID，因为 inspect 命令可能不支持 ID
  --     cmd = { 'docker', 'inspect', entry.name }
  --     lc.system(cmd, function(inspect_out)
  --       if inspect_out.code == 0 then
  --         local success, data = pcall(lc.json.decode, inspect_out.stdout)
  --         if success and type(data) == 'table' and #data > 0 then
  --           local img = data[1]
  --           local lines = {
  --             'Image: ' .. (entry.name or 'unknown'),
  --             '',
  --             'ID: ' .. (entry.id or 'unknown'),
  --           }
  --           if entry.created then table.insert(lines, 'Created: ' .. entry.created) end
  --           if entry.size then table.insert(lines, 'Size: ' .. entry.size) end
  --           local arch = img.Architecture or img.architecture
  --           if arch then table.insert(lines, 'Architecture: ' .. arch) end
  --           local os = img.Os or img.os
  --           if os then table.insert(lines, 'OS: ' .. os) end
  --           local config = img.Config or img.config
  --           if config and config.Cmd then
  --             local cmd_str = type(config.Cmd) == 'table' and table.concat(config.Cmd, ' ') or config.Cmd
  --             table.insert(lines, 'Command: ' .. (cmd_str or ''))
  --           end
  --           cb(table.concat(lines, '\n'))
  --           return
  --         end
  --       end
  --       cb 'Failed to get image details'
  --     end)
  --     return
  --   elseif resource_type == 'volume' then
  --     -- 显示卷详细信息
  --     local lines = {
  --       'Volume: ' .. (entry.name or 'unknown'),
  --       '',
  --       'Driver: ' .. (entry.driver or ''),
  --     }
  --     if entry.scope then table.insert(lines, 'Scope: ' .. entry.scope) end
  --     if entry.mountpoint then table.insert(lines, 'Mountpoint: ' .. entry.mountpoint) end
  --     cb(table.concat(lines, '\n'))
  --     return
  --   elseif resource_type == 'network' then
  --     -- 显示网络详细信息
  --     cmd = { 'docker', 'network', 'inspect', entry.id }
  --     lc.system(cmd, function(inspect_out)
  --       if inspect_out.code == 0 then
  --         local success, data = pcall(lc.json.decode, inspect_out.stdout)
  --         if success and type(data) == 'table' and #data > 0 then
  --           local net = data[1]
  --           local lines = {
  --             'Network: ' .. (entry.name or 'unknown'),
  --             '',
  --             'ID: ' .. (entry.id or 'unknown'),
  --             'Driver: ' .. (entry.driver or ''),
  --           }
  --           if entry.scope then table.insert(lines, 'Scope: ' .. entry.scope) end
  --           if entry.subnet then table.insert(lines, 'Subnet: ' .. entry.subnet) end
  --           if entry.gateway then table.insert(lines, 'Gateway: ' .. entry.gateway) end
  --           local ipam = net.IPAM or net.ipam
  --           if ipam and ipam.Config and #ipam.Config > 0 then
  --             for _, config in ipairs(ipam.Config) do
  --               if config.Subnet or config.subnet then
  --                 table.insert(lines, 'Subnet: ' .. (config.Subnet or config.subnet))
  --               end
  --             end
  --           end
  --           cb(table.concat(lines, '\n'))
  --           return
  --         end
  --       end
  --       cb 'Failed to get network details'
  --     end)
  --     return
  --   end
  -- end

  -- cb 'No preview available'
end

return M
