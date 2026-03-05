local M = {}

-- 辅助函数：获取当前选中的单元信息
local function get_selected_unit()
  local entry = lc.api.page_get_hovered()
  local path = lc.api.get_current_path()

  if not entry or not entry.unit or #path < 2 then return nil end

  return {
    scope = path[1],
    unit = entry.unit,
  }
end

local function do_unit_action(action_name)
  local unit_info = get_selected_unit()
  if not unit_info then
    lc.notify 'Please select a unit first'
    return
  end

  local cmd = {}
  if unit_info.scope == 'system' then cmd = { 'sudo' } end

  if action_name == 'follow' then
    cmd = lc.tbl_extend(cmd, { 'journalctl', '--' .. unit_info.scope, '-xef', '--unit=' .. unit_info.unit })
  else
    cmd = lc.tbl_extend(cmd, { 'systemctl', '--' .. unit_info.scope, action_name, unit_info.unit })
  end

  lc.interactive(cmd, { wait_confirm = function(exit_code) return exit_code ~= 0 end }, function(exit_code)
    if exit_code == 0 then
      lc.notify(action_name .. ' for ' .. unit_info.unit .. ' successfully')
      lc.cmd 'reload'
    else
      lc.notify(action_name .. ' for ' .. unit_info.unit .. ' failed')
    end
  end)
end

function M.restart() do_unit_action 'restart' end

function M.start() do_unit_action 'start' end

function M.stop() do_unit_action 'stop' end

function M.enable() do_unit_action 'enable' end

function M.reload() do_unit_action 'reload' end

function M.follow() do_unit_action 'follow' end

function M.edit() do_unit_action 'edit' end

function M.show() do_unit_action 'show' end

function M.cat() do_unit_action 'cat' end

function M.select_action()
  local path = lc.api.get_current_path()
  if #path < 2 then
    lc.cmd 'enter'
  else
    lc.select({
      prompt = 'Select an action',
      options = { 'follow', 'restart', 'start', 'stop', 'enable', 'edit', 'show', 'cat', 'reload' },
    }, function(choice)
      if choice then require('systemd.action')[choice]() end
    end)
  end
end

return M
