---@class lc.api
local api = {}

---@class PageEntry
---@field key string The unique key for the entry
---@field display? string|Span The display text or Span widget
---@field keymap? table<string, fun()|{callback: fun(), desc?: string}> Entry-local keymap table, resolved from the entry/metatable and preferred over global keymaps when matched
---@field preview? fun(self: PageEntry, cb: fun(widget: string|Span|Text|Line)) Entry-local preview callback, preferred over plugin.preview when present
---@field [string] any Additional custom fields

---Set the entries for a page
---@param path string[]|nil The page path, or nil for the current page
---@param entries PageEntry[]|nil The list of page entries, or nil to clear the page
function api.set_entries(path, entries) return _lc.api.set_entries(path, entries) end

---Get the currently hovered entry
---@return PageEntry? entry The hovered entry or nil
function api.get_hovered() return _lc.api.get_hovered() end

---Set hovered entry by full path
---@param path string[] The full path including the entry key
function api.set_hovered(path) return _lc.api.set_hovered(path) end

---Get the full entry list for a page before filtering
---@param path string[]|nil The page path, or nil for the current page
---@return PageEntry[]|nil entries The page entries
function api.get_entries(path) return _lc.api.get_entries(path) end

---Set the preview panel content
---@param path string[]|nil The hovered entry path, or nil for the current hovered entry
---@param widget string|Span|Text|Line|nil The widget to display, or nil to clear the preview
function api.set_preview(path, widget) return _lc.api.set_preview(path, widget) end

---Navigate to a specific path
---@param path string[] The path as an array of strings
function api.go_to(path) return _lc.api.go_to(path) end

---Clear the cached page for a specific path so the next navigation reloads it
---@param path string[] The path as an array of strings
function api.clear_page_cache(path) return _lc.api.clear_page_cache(path) end

---Get the current navigation path
---@return string[] path The current path
function api.get_current_path() return _lc.api.get_current_path() end

---Get the full path of the currently hovered entry
---@return string[]|nil path The full path or nil
function api.get_hovered_path() return _lc.api.get_hovered_path() end

---Get command line arguments
---@return string[] args Command line arguments (first element is program name)
function api.argv() return _lc.api.argv() end

---Set the filter string for the current page
---The page entries will be filtered based on this string
---If empty string, no filter is applied (show all entries)
---@param filter string The filter string to apply
function api.set_filter(filter) _lc.api.set_filter(filter) end

---Get the current filter string for the current page
---@return string filter The current filter string, or empty string if none
function api.get_filter() return _lc.api.get_filter() end

---@class AvailableKeymap
---@field key string
---@field desc? string
---@field callback fun()
---@field source "entry"|"global"

---Get all currently available keymaps in the current context
---Entry-local keymaps are returned before global keymaps
---@return AvailableKeymap[]
function api.get_available_keymaps() return _lc.api.get_available_keymaps() end

lc.api = api
lc.hook = lc.hook or {}

---Append a hook callback to be called before reload command
---@param callback fun() The callback function to execute before reload
function lc.hook.pre_reload(callback) _lc.api.append_hook_pre_reload(callback) end

---Append a hook callback to be called before quit command
---@param callback fun() The callback function to execute before quit
function lc.hook.pre_quit(callback) _lc.api.append_hook_pre_quit(callback) end

---Append a hook callback to be called after entering a page
---@param callback fun(ctx: {path: string[]}) The callback function to execute
function lc.hook.post_page_enter(callback) _lc.api.append_hook_post_page_enter(callback) end

---Send an internal command to Rust
---@param command string The command string (e.g., "quit", "reload", "scroll_by 1")
function lc.cmd(command) return _lc.cmd(command) end

---Execute a function after a delay
---@param callback fun() The function to execute
---@param delay_ms number Delay in milliseconds
function lc.defer_fn(callback, delay_ms) return _lc.defer_fn(callback, delay_ms) end
