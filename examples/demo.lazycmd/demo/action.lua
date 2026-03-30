local M = {}
function M.open_file(entry)
  entry = entry or lc.api.page_get_hovered()
  if not entry or entry.kind ~= 'file' or not entry.path then return end
  lc.system.open(entry.path)
end

return M
