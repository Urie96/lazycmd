---@class lc.api
local api = {}

---@class PageEntry
---@field key string The unique key for the entry
---@field display? string|Span The display text or Span widget
---@field [string] any Additional custom fields

---Set the entries for the current page
---@param entries PageEntry[] The list of page entries
function api.page_set_entries(entries) return _lc.api.page_set_entries(entries) end

---Get the currently hovered entry
---@return PageEntry? entry The hovered entry or nil
function api.page_get_hovered() return _lc.api.page_get_hovered() end

---Set the preview panel content
---@param widget string|Span|Text|Line The widget to display in the preview panel
function api.page_set_preview(widget) return _lc.api.page_set_preview(widget) end

---Navigate to a specific path
---@param path string[] The path as an array of strings
function api.go_to(path) return _lc.api.go_to(path) end

---Get the current navigation path
---@return string[] path The current path
function api.get_current_path() return _lc.api.get_current_path() end

---Get the full path of the currently hovered entry
---@return string[]|nil path The full path or nil
function api.get_hovered_path() return _lc.api.get_hovered_path() end

---Get command line arguments
---@return string[] args Command line arguments (first element is program name)
function api.argv() return _lc.api.argv() end

---Append a hook callback to be called before reload command
---@param callback fun() The callback function to execute before reload
function api.append_hook_pre_reload(callback) _lc.api.append_hook_pre_reload(callback) end

---Append a hook callback to be called before quit command
---@param callback fun() The callback function to execute before quit
function api.append_hook_pre_quit(callback) _lc.api.append_hook_pre_quit(callback) end

---Set the filter string for the current page
---The page entries will be filtered based on this string
---If empty string, no filter is applied (show all entries)
---@param filter string The filter string to apply
function api.set_filter(filter) _lc.api.set_filter(filter) end

---Get the current filter string for the current page
---@return string filter The current filter string, or empty string if none
function api.get_filter() return _lc.api.get_filter() end

lc.api = api

---Send an internal command to Rust
---@param command string The command string (e.g., "quit", "reload", "scroll_by 1")
function lc.cmd(command) return _lc.cmd(command) end

---Execute a function after a delay
---@param callback fun() The function to execute
---@param delay_ms number Delay in milliseconds
function lc.defer_fn(callback, delay_ms) return _lc.defer_fn(callback, delay_ms) end
