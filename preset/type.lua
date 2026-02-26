-- Type definitions for lazycmd Lua API
-- This file provides type hints for Lua LSP/EmmyLua

-- ============================================
-- Global: lc
-- ============================================

---@class lc
lc = {}

-- ============================================
-- lc.api - Page management and system functions
-- ============================================

---@class lc.api
local api = {}

---Set the entries for the current page
---@param entries PageEntry[] The list of page entries
function api.page_set_entries(entries) end

---Get the currently hovered entry
---@return PageEntry|nil entry The hovered entry or nil
function api.page_get_hovered() end

---Set the preview panel content
---@param widget Renderable The widget to display in the preview panel
function api.page_set_preview(widget) end

---Navigate to a specific path
---@param path string[] The path as an array of strings
function api.go_to(path) end

---Get the current navigation path
---@return string[] path The current path
function api.get_current_path() end

---Get the full path of the currently hovered entry
---@return string[]|nil path The full path or nil
function api.get_hovered_path() end

---Get command line arguments
---@return string[] args Command line arguments (first element is program name)
function api.argv() end

lc.api = api

-- ============================================
-- lc.fs - File system operations
-- ============================================

---@class lc.fs
local fs = {}

---Read directory synchronously
---@param path string The directory path to read
---@return table[] entries List of directory entries
---@return string|nil error Error message if failed
function fs.read_dir_sync(path) end

lc.fs = fs

-- ============================================
-- lc.keymap - Keyboard mapping
-- ============================================

---@class lc.keymap
local keymap = {}

---Set a key mapping for a specific mode
---@param mode Mode The mode (e.g., "main", "input")
---@param key string The key sequence (e.g., "ctrl-d", "down", "j")
---@param callback string|fun() The command string or callback function
function keymap.set(mode, key, callback) end

lc.keymap = keymap

-- ============================================
-- lc.path - Path manipulation
-- ============================================

---@class lc.path
local path = {}

---Split a path into components
---@param path string|PathBuf The path to split
---@return string[] components The path components
function path.split(path) end

---Join path components into a single path
---@param components string[]|OsString[] The path components
---@return PathBuf path The joined path
function path.join(components) end

lc.path = path

-- ============================================
-- lc.system - Execute external commands
-- ============================================

---@class CommandOutput
---@field code number Exit code
---@field stdout string Standard output
---@field stderr string Standard error

---Execute an external command asynchronously
---@param cmd string[] The command and its arguments
---@param callback fun(output: CommandOutput) Callback function called on completion
function lc.system(cmd, callback) end

-- ============================================
-- lc.defer_fn - Schedule delayed execution
-- ============================================

---Execute a function after a delay
---@param callback fun() The function to execute
---@param delay_ms number Delay in milliseconds
function lc.defer_fn(callback, delay_ms) end

-- ============================================
-- lc.cmd - Send internal commands
-- ============================================

---Send an internal command to Rust
---@param command string The command string (e.g., "quit", "reload", "scroll_by 1")
function lc.cmd(command) end

-- ============================================
-- lc.on_event - Register event hooks
-- ============================================

---@class EventHook
---@field EnterPost string Fired after entering a directory
---@field HoverPost string Fired after changing selection

---Register a callback for an event
---@param event_name EventHook The event name
---@param callback fun() The callback function
function lc.on_event(event_name, callback) end

-- ============================================
-- lc.split - String splitting utility
-- ============================================

---Split a string by a separator
---@param s string The string to split
---@param sep string The separator
---@return string[] parts The split parts
function lc.split(s, sep) end

-- ============================================
-- lc.notify - Display notifications
-- ============================================

---Print a notification message
---@vararg any Values to print
function lc.notify(...) end

-- ============================================
-- lc.inspect - Pretty print values
-- ============================================

---Convert a value to a pretty-printed string
---@param value any The value to inspect
---@return string The pretty-printed representation
function lc.inspect(value) end

-- ============================================
-- lc.tbl_map - Map function over table
-- ============================================

---Map a function over table values
---@param func fun(value: any): any The mapping function
---@param t table The table to map over
---@return table The new table with mapped values
function lc.tbl_map(func, t) end

-- ============================================
-- Type aliases
-- ============================================

---@alias Mode "main"|"input"

---@class PageEntry
---@field key string The unique key for the entry
---@field is_dir? boolean Whether this is a directory
---@field display? string|Span The display text or Span widget
---@field pid? number Process ID (for process plugin)
---@field [string] any Additional custom fields

---@class Renderable
---A renderable widget (string, Text, or Span)

---@class Text
---A TUI Text widget

---@class Span
---A TUI Span widget

---@class PathBuf
---A path buffer

---@class OsString
---An OS string

-- ============================================
-- String extensions
-- ============================================

---Set text color for display
---@param color string Color name (e.g., "blue", "red", "green")
---@return Span A colored span widget
function string:fg(color) end

---Parse ANSI escape sequences into a TUI Text widget
---@return Text A Text widget with parsed ANSI codes
function string:ansi() end

---Split string by separator
---@param sep string The separator
---@return string[] The split parts
function string:split(sep) end
