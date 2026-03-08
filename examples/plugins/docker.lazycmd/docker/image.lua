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

local function docker_image_list()
  local cmd = { 'docker', 'image', 'ls', '--format', '{{json .}}' }

  return exec(cmd):next(function(stdout)
    local images = lc.tbl_map(function(line)
      local success, data = pcall(lc.json.decode, line)
      assert(success and type(data) == 'table', 'Failed to parse JSON output: ' .. line)

      return {
        id = data.ID,
        repository = data.Repository,
        tag = data.Tag,
        created_since = data.CreatedSince,
        size = data.Size,
        digest = data.Digest,
        created_at = data.CreatedAt,
      }
    end, stdout:trim():split '\n')

    return images
  end)
end

local function docker_inspect_image(image_id)
  local cmd = { 'docker', 'image', 'inspect', image_id }

  return exec(cmd):next(function(stdout)
    local success, data = pcall(lc.json.decode, stdout)
    assert(success and type(data) == 'table' and #data > 0, 'Failed to parse JSON output: ' .. stdout)
    return data[1]
  end)
end

function M.list(cb)
  docker_image_list()
    :next(function(images)
      -- 按创建时间排序（新的在前）
      table.sort(images, function(a, b)
        local a_time = a.created_at or ''
        local b_time = b.created_at or ''
        return a_time > b_time
      end)

      local entries = lc.tbl_map(function(image)
        local display_parts = {
          (image.repository .. ':' .. image.tag):fg 'yellow',
          ' ',
          image.size:fg 'cyan',
        }

        return {
          key = image.id,
          display = lc.style.line(display_parts),
          image = image,
        }
      end, images)

      local lines = lc.tbl_map(function(line) return line.display end, entries)

      lc.style.align_columns(lines)

      cb(entries)
    end)
    :catch(function(err) lc.notify('Failed to list images: ' .. err) end)
end

function M.preview(entry, cb)
  local detail_area = docker_inspect_image(entry.image.id)
    :next(function(detail)
      local image = entry.image
      local config = detail.Config or {}

      local lines = {
        lc.style.line({ '🆔 ID: ', image.id or 'unknown' }):fg 'bright_magenta',
        lc.style.line({ '📅 Created: ', image.created_since or 'unknown' }):fg 'green',
      }

      -- 添加镜像配置信息
      if config.Cmd and #config.Cmd > 0 then
        table.insert(lines, lc.style.line({ '⌨️ Cmd: ', table.concat(config.Cmd, ' ') }):fg 'blue')
      end

      if config.Entrypoint and #config.Entrypoint > 0 then
        table.insert(lines, lc.style.line({ '🚪 Entrypoint: ', table.concat(config.Entrypoint, ' ') }):fg 'yellow')
      end

      if config.WorkingDir then
        table.insert(lines, lc.style.line({ '📁 WorkingDir: ', config.WorkingDir }):fg 'bright_blue')
      end

      local arch = detail.Architecture
      if arch then table.insert(lines, lc.style.line({ '🖥️ Architecture: ', arch }):fg 'bright_green') end

      local os_type = detail.Os
      if os_type then table.insert(lines, lc.style.line({ '💻 OS: ', os_type }):fg 'bright_green') end

      if config.ExposedPorts then
        local ports = {}
        for port, _ in pairs(config.ExposedPorts) do
          table.insert(ports, port)
        end
        table.insert(lines, lc.style.line({ '🔌 Exposed Ports: ', table.concat(ports, ', ') }):fg 'bright_magenta')
      end

      if config.Env and #config.Env > 0 then
        table.insert(lines, lc.style.line({ '🌎 Environment:' }):fg 'cyan')
        for _, env in ipairs(config.Env) do
          table.insert(lines, ('    ' .. env):fg 'cyan')
        end
      end

      lc.style.align_columns(lines)
      return lines
    end)
    :catch(function(err)
      lc.notify('Failed to get image details: ' .. err)
      return { 'Failed to get image details' }
    end)

  -- 获取使用该镜像的容器
  local container_area = exec({ 'docker', 'ps', '-a', '--format', '{{json .}}' })
    :next(function(stdout)
      local containers = {}
      for line in stdout:trim():gmatch '[^\n]+' do
        local success, data = pcall(lc.json.decode, line)
        if success and type(data) == 'table' then
          -- 检查容器是否使用当前镜像
          if
            data.Image == entry.image.repository .. ':' .. entry.image.tag
            or data.Image == entry.image.id
            or data.Image:sub(1, entry.image.id:len()) == entry.image.id
          then
            table.insert(containers, data)
          end
        end
      end

      if #containers > 0 then
        local lines = {
          ' ',
          ('Containers using this image (' .. #containers .. '):'):fg 'yellow',
        }
        for _, container in ipairs(containers) do
          local color = container.State == 'running' and 'green' or 'red'
          table.insert(lines, lc.style.line { '  • ', container.Names:fg(color), ' ', container.State:fg 'white' })
        end
        return lines
      else
        return { ' ', ('No containers using this image'):fg 'gray' }
      end
    end)
    :catch(function(err) lc.notify('Failed to get container list' .. err) end)

  local history_area = exec({ 'docker', 'image', 'history', entry.image.id, '--no-trunc', '--format', '{{json .}}' })
    :next(function(stdout)
      local lines = {
        ' ',
        ' ',
        lc.style.line { 'SIZE', ' ', 'COMMAND' },
      }
      for _, line in ipairs(stdout:trim():split '\n') do
        local success, data = pcall(lc.json.decode, line)
        if success and type(data) == 'table' then
          local created_by = data.CreatedBy or ''
          table.insert(lines, lc.style.line { data.Size:fg 'cyan', ' ', created_by })
        end
      end
      lc.style.align_columns(lines)
      return lines
    end)
    :catch(function(err) lc.notify('Failed to get image history' .. err) end)

  promise.all({ detail_area, container_area, history_area }):next(function(results)
    local lines = results[1]
    lc.list_extend(lines, results[2])
    lc.list_extend(lines, results[3])
    cb(lc.style.text(lines))
  end)
end

-- 辅助函数：获取当前选中的镜像信息
local function get_selected_image()
  local entry = lc.api.page_get_hovered()
  if not entry or not entry.image then return nil end
  return entry
end

function M.select_action()
  local entry = get_selected_image()
  if not entry then
    lc.notify 'Please select an image first'
    return
  end

  local image = entry.image
  local options = {
    { value = 'inspect', display = lc.style.line { ('🔍 Inspect'):fg 'cyan' } },
    { value = 'remove', display = lc.style.line { ('🗑️ Remove'):fg 'red' } },
    { value = 'pull', display = lc.style.line { ('⬇️ Pull/Update'):fg 'blue' } },
    { value = 'save', display = lc.style.line { ('💾 Save to file'):fg 'green' } },
  }

  lc.select({
    prompt = 'Select an action',
    options = options,
  }, function(choice)
    if choice then M[choice](image) end
  end)
end

function M.inspect(image)
  exec({ 'docker', 'image', 'inspect', image.id })
    :next(function(stdout) lc.api.page_set_preview(lc.style.highlight(stdout, 'json')) end)
    :catch(function(stderr) lc.notify('Failed to inspect image: ' .. stderr) end)
end

function M.pull(image)
  local image_ref = image.repository .. ':' .. image.tag
  lc.notify('Pulling image: ' .. image_ref)
  lc.interactive { 'docker', 'pull', image_ref }
end

function M.remove(image)
  local image_ref = image.repository .. ':' .. image.tag
  lc.confirm {
    prompt = 'Remove image: ' .. image_ref .. '?',
    on_confirm = function()
      lc.notify('Removing image: ' .. image_ref)
      lc.system({ 'docker', 'rmi', image.id }, function(out)
        if out.code == 0 then
          lc.notify 'Image removed successfully'
          lc.cmd 'reload'
        else
          lc.notify('Failed to remove image: ' .. out.stderr)
        end
      end)
    end,
  }
end

function M.save(image)
  local filename = image.repository:gsub('/', '-') .. '-' .. image.tag .. '.tar'
  lc.confirm {
    prompt = 'Save image to file: ' .. filename .. '?',
    on_confirm = function()
      lc.notify('Saving image to file: ' .. filename)
      lc.system({ 'docker', 'save', image.id, '-o', filename }, function(out)
        if out.code == 0 then
          lc.notify 'Image saved successfully'
        else
          lc.notify('Failed to save image: ' .. out.stderr)
        end
      end)
    end,
  }
end

return M
