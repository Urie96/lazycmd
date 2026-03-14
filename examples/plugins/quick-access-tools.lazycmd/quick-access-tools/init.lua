-- t: A lazycmd plugin for text transformations and utilities

local M = {}

local all_tools = {}

function M.setup()
  local modules = {
    'quick-access-tools.formatter',
    'quick-access-tools.date',
  }

  for _, module in ipairs(modules) do
    local tools = require(module)
    lc.list_extend(all_tools, tools)
  end

  -- Keymap: y to copy result
  lc.keymap.set('main', 'y', function()
    local entry = lc.api.page_get_hovered()
    if entry and entry.on_copy then entry.on_copy(entry) end
  end)

  -- Keymap: <enter> to execute tool
  lc.keymap.set('main', '<enter>', function()
    local entry = lc.api.page_get_hovered()
    if entry and entry.on_enter then entry.on_enter() end
  end)
end

function M.list(_, cb) cb(all_tools) end

function M.preview(entry, cb)
  cb(lc.style.text {
    (entry.description or 'No description'):fg 'green',
    ' ',
    entry.on_enter and ('Press Enter to execute'):fg 'darkgray',
    entry.on_copy and ('Press y to copy result to clipboard'):fg 'darkgray',
  })
end

return M
