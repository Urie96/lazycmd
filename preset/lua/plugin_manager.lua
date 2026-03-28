--- Plugin Manager: core logic for installing, updating, and managing plugins
--- Handles GitHub repo parsing, git operations, and lock file management.

local pm = {}

-- Plugin data directory and lock file path
pm.data_dir = os.getenv('HOME') .. '/.local/share/lazycmd/plugins'
pm.lock_file = os.getenv('HOME') .. '/.local/share/lazycmd/plugins.lock'

local function is_absolute_path(path)
  return path:match '^/' or path:match '^%a:[/\\]'
end

local function resolve_local_dir(dir)
  if type(dir) ~= 'string' or dir == '' then
    error 'plugin dir must be a non-empty string'
  end

  if dir:find '://' then
    error('plugin dir must be a relative or absolute path: ' .. dir)
  end

  if is_absolute_path(dir) then return dir end

  local base_dir = os.getenv('HOME') .. '/.config/lazycmd'
  return base_dir .. '/' .. dir
end

local function plugin_name_from_dir(dir)
  local normalized = dir:gsub('[\\/]+$', '')
  local basename = normalized:match '([^/\\]+)$' or normalized
  return basename:match('^(.+)%.lazycmd$') or basename
end

--- Parse a plugin spec into a normalized structure.
--- Supports four input formats:
---   1. String: 'owner/plugin.lazycmd' or 'local-plugin'
---   2. Table with single string: { 'owner/plugin.lazycmd' }
---   3. Local dir: { dir='plugins/my-plugin.lazycmd' }
---   4. Full table: { 'owner/plugin.lazycmd', branch='main', config=fn }
--- @param spec string|table Plugin declaration
--- @return table|nil Parsed spec with fields: name, repo, branch, tag, commit, config, url, install_path, is_remote, dir
function pm.parse_plugin_spec(spec)
  local source
  local dir

  if type(spec) == 'string' then
    source = spec
  elseif type(spec) == 'table' then
    source = spec[1]
    dir = spec.dir
  else
    return nil
  end

  if not source and not dir then return nil end

  local name
  if dir then
    name = source or plugin_name_from_dir(dir)
  elseif source:find('/') then
    local repo_name = source:match('^[^/]+/(.+)$')
    name = repo_name:match('^(.+)%.lazycmd$') or repo_name
  else
    name = source
  end

  local branch, tag, commit, config_fn
  if type(spec) == 'table' then
    branch = spec.branch
    tag = spec.tag
    commit = spec.commit
    config_fn = spec.config
    if spec.dependencies ~= nil then
      error("plugin spec no longer supports 'dependencies'; list all plugins directly in lc.config.plugins")
    end
  end

  if not config_fn then
    config_fn = function()
      local ok, mod = pcall(require, name)
      if ok and mod and mod.setup then
        mod.setup()
      end
    end
  end

  local result = {
    name = name,
    branch = branch,
    tag = tag,
    commit = commit,
    config = config_fn,
  }

  if dir then
    result.dir = resolve_local_dir(dir)
    result.is_remote = false
  elseif source:find('/') then
    result.repo = source
    result.url = 'https://github.com/' .. source .. '.git'
    result.install_path = pm.data_dir .. '/' .. source:match('^[^/]+/(.+)$')
    result.is_remote = true
  else
    result.is_remote = false
  end

  return result
end

--- Flatten a plugin list into parsed specs (no duplicates, preserves first occurrence order).
--- @param plugins table Array of plugin specs (raw, before parsing)
--- @return table Array of parsed specs
function pm.flatten_plugins(plugins)
  local seen = {}
  local result = {}

  for _, p in ipairs(plugins) do
    local spec = pm.parse_plugin_spec(p)
    if spec and not seen[spec.name] then
      seen[spec.name] = true
      result[#result + 1] = spec
    end
  end

  return result
end

--- Read the lock file.
--- @return table Lock data: { plugin_name = { commit=..., branch=..., ... }, ... }
function pm.read_lock()
  local content, err = lc.fs.read_file_sync(pm.lock_file)
  if err or not content or content == '' then
    return {}
  end
  local ok, data = pcall(lc.json.decode, content)
  if ok and type(data) == 'table' then
    return data
  end
  return {}
end

--- Write lock data to the lock file.
--- @param lock_data table Lock data to write
function pm.write_lock(lock_data)
  lc.fs.mkdir(pm.data_dir)
  local content = lc.json.encode(lock_data)
  lc.fs.write_file_sync(pm.lock_file, content)
end

--- Check if a remote plugin is installed (directory exists).
--- @param spec table Parsed plugin spec
--- @return boolean
function pm.is_installed(spec)
  if not spec.is_remote then return true end
  if not spec.install_path then return false end
  local stat = lc.fs.stat(spec.install_path)
  return stat.exists and stat.is_dir
end

--- Update the lock file entry for a single plugin (async: gets current HEAD).
--- @param spec table Parsed plugin spec
--- @param callback function|nil Called when done
function pm.update_lock_for_plugin(spec, callback)
  lc.system({ 'git', '-C', spec.install_path, 'rev-parse', 'HEAD' }, function(out)
    if out.code == 0 then
      local lock = pm.read_lock()
      lock[spec.name] = {
        repo = spec.repo,
        commit = out.stdout:trim(),
        branch = spec.branch,
        tag = spec.tag,
        url = spec.url,
      }
      pm.write_lock(lock)
    end
    if callback then callback() end
  end)
end

--- Install a single plugin via git clone.
--- @param spec table Parsed plugin spec
--- @param callback function|nil Called with (boolean success)
function pm.install(spec, callback)
  if not spec.is_remote then
    if callback then callback(true) end
    return
  end

  lc.fs.mkdir(pm.data_dir)

  local cmd = { 'git', 'clone' }

  if spec.tag then
    -- Clone specific tag
    table.insert(cmd, '--branch')
    table.insert(cmd, spec.tag)
    table.insert(cmd, '--single-branch')
    table.insert(cmd, '--depth')
    table.insert(cmd, '1')
  elseif spec.branch then
    -- Clone specific branch
    table.insert(cmd, '--branch')
    table.insert(cmd, spec.branch)
    table.insert(cmd, '--single-branch')
    table.insert(cmd, '--depth')
    table.insert(cmd, '1')
  elseif not spec.commit then
    -- No constraint: shallow clone default branch
    table.insert(cmd, '--depth')
    table.insert(cmd, '1')
  end

  table.insert(cmd, spec.url)
  table.insert(cmd, spec.install_path)

  lc.system(cmd, function(out)
    if out.code ~= 0 then
      lc.log('error', 'Failed to install {}: {}', spec.name, out.stderr:trim())
      lc.notify(lc.style.line({
        lc.style.span('✗ '):fg('red'),
        lc.style.span('Failed to install ' .. spec.name),
      }))
      if callback then callback(false) end
      return
    end

    if spec.commit then
      -- Need full history to checkout a specific commit
      lc.system({ 'git', '-C', spec.install_path, 'fetch', '--unshallow' }, function()
        lc.system({ 'git', '-C', spec.install_path, 'checkout', spec.commit }, function(out2)
          if out2.code ~= 0 then
            lc.log('error', 'Failed to checkout commit {} for {}: {}', spec.commit, spec.name, out2.stderr:trim())
          end
          pm.update_lock_for_plugin(spec, function()
            if callback then callback(out2.code == 0) end
          end)
        end)
      end)
    else
      pm.update_lock_for_plugin(spec, function()
        if callback then callback(true) end
      end)
    end
  end)
end

--- Update a single plugin within its constraints.
--- @param spec table Parsed plugin spec
--- @param callback function|nil Called with (boolean success)
function pm.update(spec, callback)
  if not spec.is_remote then
    if callback then callback(false) end
    return
  end

  if spec.commit then
    -- Commit-pinned plugins cannot be updated
    lc.notify(lc.style.line({
      lc.style.span('⊘ '):fg('yellow'),
      lc.style.span(spec.name .. ' is pinned to commit ' .. spec.commit:sub(1, 7)),
    }))
    if callback then callback(false) end
    return
  end

  if not pm.is_installed(spec) then
    -- Not installed yet, install instead
    pm.install(spec, callback)
    return
  end

  local install_path = spec.install_path

  -- git fetch
  lc.system({ 'git', '-C', install_path, 'fetch', '--tags', '--force' }, function(out)
    if out.code ~= 0 then
      lc.log('error', 'Failed to fetch {}: {}', spec.name, out.stderr:trim())
      lc.notify(lc.style.line({
        lc.style.span('✗ '):fg('red'),
        lc.style.span('Failed to fetch ' .. spec.name),
      }))
      if callback then callback(false) end
      return
    end

    local function on_done(out2)
      if out2.code ~= 0 then
        lc.log('error', 'Failed to update {}: {}', spec.name, out2.stderr:trim())
        lc.notify(lc.style.line({
          lc.style.span('✗ '):fg('red'),
          lc.style.span('Failed to update ' .. spec.name),
        }))
        pm.update_lock_for_plugin(spec, function()
          if callback then callback(false) end
        end)
      else
        pm.update_lock_for_plugin(spec, function()
          if callback then callback(true) end
        end)
      end
    end

    if spec.tag then
      -- Tag constraint: checkout the tag (tracks the exact commit the tag points to)
      lc.system({ 'git', '-C', install_path, 'checkout', 'tags/' .. spec.tag }, on_done)
    elseif spec.branch then
      -- Branch constraint: reset to the latest of that branch
      lc.system({ 'git', '-C', install_path, 'reset', '--hard', 'origin/' .. spec.branch }, on_done)
    else
      -- No constraint: reset to default remote branch
      -- First, determine the default remote branch
      lc.system({ 'git', '-C', install_path, 'symbolic-ref', 'refs/remotes/origin/HEAD' }, function(ref_out)
        local default_ref = 'origin/HEAD'
        if ref_out.code == 0 then
          -- Extract branch name from refs/remotes/origin/main -> origin/main
          local ref = ref_out.stdout:trim()
          default_ref = ref:gsub('^refs/remotes/', '')
        end
        lc.system({ 'git', '-C', install_path, 'reset', '--hard', default_ref }, on_done)
      end)
    end
  end)
end

--- Restore a plugin to the version recorded in the lock file.
--- @param spec table Parsed plugin spec
--- @param lock_entry table|nil Lock file entry for this plugin
--- @param callback function|nil Called with (boolean success)
function pm.restore(spec, lock_entry, callback)
  if not spec.is_remote then
    if callback then callback(false) end
    return
  end

  if not lock_entry or not lock_entry.commit then
    lc.notify(lc.style.line({
      lc.style.span('⊘ '):fg('yellow'),
      lc.style.span(spec.name .. ': no lock entry found'),
    }))
    if callback then callback(false) end
    return
  end

  local install_path = spec.install_path
  local stat = lc.fs.stat(install_path)

  if not stat.exists then
    -- Clone then checkout to locked commit
    lc.system({ 'git', 'clone', spec.url, install_path }, function(out)
      if out.code == 0 then
        lc.system({ 'git', '-C', install_path, 'checkout', lock_entry.commit }, function(out2)
          if callback then callback(out2.code == 0) end
        end)
      else
        lc.log('error', 'Failed to clone {} for restore: {}', spec.name, out.stderr:trim())
        if callback then callback(false) end
      end
    end)
  else
    -- Fetch then checkout to locked commit
    lc.system({ 'git', '-C', install_path, 'fetch' }, function()
      lc.system({ 'git', '-C', install_path, 'checkout', lock_entry.commit }, function(out2)
        if out2.code == 0 then
          lc.system({ 'git', '-C', install_path, 'reset', '--hard', lock_entry.commit }, function(out3)
            if callback then callback(out3.code == 0) end
          end)
        else
          if callback then callback(false) end
        end
      end)
    end)
  end
end

--- Check whether a plugin has a newer version available (within constraints).
--- @param spec table Parsed plugin spec
--- @param callback function Called with (boolean has_update, string|nil remote_info)
function pm.check_update(spec, callback)
  if not spec.is_remote or not pm.is_installed(spec) then
    callback(false, nil)
    return
  end

  if spec.commit then
    -- Commit-pinned: never has updates
    callback(false, nil)
    return
  end

  local install_path = spec.install_path

  -- Fetch latest
  lc.system({ 'git', '-C', install_path, 'fetch', '--tags', '--force' }, function(fetch_out)
    if fetch_out.code ~= 0 then
      callback(false, nil)
      return
    end

    -- Get local HEAD
    lc.system({ 'git', '-C', install_path, 'rev-parse', 'HEAD' }, function(local_out)
      if local_out.code ~= 0 then
        callback(false, nil)
        return
      end
      local local_commit = local_out.stdout:trim()

      -- Determine remote ref
      local remote_ref
      if spec.tag then
        remote_ref = 'tags/' .. spec.tag
      elseif spec.branch then
        remote_ref = 'origin/' .. spec.branch
      else
        remote_ref = 'origin/HEAD'
      end

      lc.system({ 'git', '-C', install_path, 'rev-parse', remote_ref }, function(remote_out)
        if remote_out.code ~= 0 then
          callback(false, nil)
          return
        end
        local remote_commit = remote_out.stdout:trim()
        local has_update = local_commit ~= remote_commit

        if has_update then
          -- Get log between local and remote
          lc.system({
            'git', '-C', install_path, 'log',
            '--oneline', '--no-decorate',
            local_commit .. '..' .. remote_commit,
          }, function(log_out)
            local info = remote_commit:sub(1, 7)
            if log_out.code == 0 and log_out.stdout:trim() ~= '' then
              local lines = log_out.stdout:trim():split('\n')
              info = info .. ' (' .. #lines .. ' new commits)'
            end
            callback(true, info)
          end)
        else
          callback(false, nil)
        end
      end)
    end)
  end)
end

--- Get the list of remote plugins from a plugins config list.
--- @param plugins table Array of plugin specs
--- @return table Array of parsed remote plugin specs
function pm.get_remote_plugins(plugins)
  local result = {}
  for _, spec in ipairs(pm.flatten_plugins(plugins or {})) do
    if spec and spec.is_remote then
      table.insert(result, spec)
    end
  end
  return result
end

--- Install a list of parsed plugin specs, skipping ones already present.
--- @param specs table Array of parsed plugin specs
--- @param callback function|nil Called with (boolean success)
function pm.install_specs(specs, callback)
  local missing = {}
  for _, spec in ipairs(specs or {}) do
    if not pm.is_installed(spec) then
      table.insert(missing, spec)
    end
  end

  if #missing == 0 then
    if callback then callback(true) end
    return
  end

  local idx = 0
  local all_ok = true
  local function install_next()
    idx = idx + 1
    if idx > #missing then
      lc.notify(lc.style.line({
        lc.style.span('✓ '):fg('green'),
        lc.style.span('Installed ' .. #missing .. ' plugin(s)'),
      }))
      if callback then callback(all_ok) end
      return
    end

    local spec = missing[idx]
    lc.notify(lc.style.line({
      lc.style.span('⟳ '):fg('cyan'),
      lc.style.span('Installing ' .. spec.name .. ' (' .. idx .. '/' .. #missing .. ')...'),
    }))
    pm.install(spec, function(success)
      all_ok = all_ok and success
      install_next()
    end)
  end

  install_next()
end

--- Install all missing remote plugins (sequentially).
--- @param plugins table Array of plugin spec tables
--- @param callback function|nil Called with (boolean success) when all done
function pm.install_missing(plugins, callback)
  pm.install_specs(pm.get_remote_plugins(plugins), callback)
end

--- Update all remote plugins (sequentially).
--- @param plugins table Array of plugin spec tables
--- @param callback function|nil Called when all done
function pm.update_all(plugins, callback)
  local remote = pm.get_remote_plugins(plugins)
  if #remote == 0 then
    if callback then callback() end
    return
  end

  local idx = 0
  local updated = 0
  local function update_next()
    idx = idx + 1
    if idx > #remote then
      lc.notify(lc.style.line({
        lc.style.span('✓ '):fg('green'),
        lc.style.span('Updated ' .. updated .. '/' .. #remote .. ' plugin(s)'),
      }))
      if callback then callback() end
      return
    end

    local spec = remote[idx]
    lc.notify(lc.style.line({
      lc.style.span('⟳ '):fg('cyan'),
      lc.style.span('Updating ' .. spec.name .. ' (' .. idx .. '/' .. #remote .. ')...'),
    }))
    pm.update(spec, function(success)
      if success then updated = updated + 1 end
      update_next()
    end)
  end

  update_next()
end

--- Restore all remote plugins from the lock file (sequentially).
--- @param plugins table Array of plugin spec tables
--- @param callback function|nil Called when all done
function pm.restore_all(plugins, callback)
  local remote = pm.get_remote_plugins(plugins)
  local lock = pm.read_lock()

  if #remote == 0 then
    if callback then callback() end
    return
  end

  local idx = 0
  local restored = 0
  local function restore_next()
    idx = idx + 1
    if idx > #remote then
      lc.notify(lc.style.line({
        lc.style.span('✓ '):fg('green'),
        lc.style.span('Restored ' .. restored .. '/' .. #remote .. ' plugin(s) from lock file'),
      }))
      if callback then callback() end
      return
    end

    local spec = remote[idx]
    local lock_entry = lock[spec.name]
    lc.notify(lc.style.line({
      lc.style.span('⟳ '):fg('cyan'),
      lc.style.span('Restoring ' .. spec.name .. ' (' .. idx .. '/' .. #remote .. ')...'),
    }))
    pm.restore(spec, lock_entry, function(success)
      if success then restored = restored + 1 end
      restore_next()
    end)
  end

  restore_next()
end

-- Attach _pm to the same underlying table that _lc points to.
-- Use the global 'lc' directly since _lc and lc reference the same table.
-- The global 'lc' was registered by Rust's lc::register() via lua.globals().raw_set("_lc", lc).
lc._pm = pm
