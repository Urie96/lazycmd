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

function M.restart()
  local unit_info = get_selected_unit()

  if not unit_info then
    lc.notify 'Please select a unit first'
    return
  end

  local cmd = { 'systemctl', unit_info.scope, 'restart', unit_info.unit }

  lc.system(cmd, function(out)
    if out.code == 0 then
      lc.notify(unit_info.unit .. ' restarted successfully')
      lc.cmd 'reload'
    else
      lc.notify('Failed to restart ' .. unit_info.unit)
    end
  end)
end

function M.start()
  local unit_info = get_selected_unit()

  if not unit_info then
    lc.notify 'Please select a unit first'
    return
  end

  local cmd = { 'systemctl', unit_info.scope, 'restart', unit_info.unit }

  lc.system(cmd, function(out)
    if out.code == 0 then
      lc.notify(unit_info.unit .. ' restarted successfully')
      lc.cmd 'reload'
    else
      lc.notify('Failed to restart ' .. unit_info.unit)
    end
  end)
end

return M
