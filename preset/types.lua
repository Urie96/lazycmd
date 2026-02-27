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
---@param widget Renderable|string The widget to display in the preview panel
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

---Read file content synchronously
---@param path string The file path to read
---@return string content The file content
---@return string|nil error Error message if failed
function fs.read_file_sync(path) end

---Write content to file synchronously
---@param path string The file path to write
---@param content string The content to write
---@return boolean success Whether the write succeeded
---@return string|nil error Error message if failed
function fs.write_file_sync(path, content) end

lc.fs = fs

-- ============================================
-- lc.http - HTTP requests
-- ============================================

---@class HttpResponse
---@field success boolean Whether the request succeeded
---@field status number HTTP status code
---@field body string Response body
---@field headers table<string, string> Response headers
---@field error string|nil Error message if failed

---@class lc.http
local http = {}

---Send a GET request
---@param url string The request URL
---@param callback fun(response: HttpResponse) Callback function
function http.get(url, callback) end

---Send a POST request
---@param url string The request URL
---@param body string Request body
---@param callback fun(response: HttpResponse) Callback function
function http.post(url, body, callback) end

---Send a PUT request
---@param url string The request URL
---@param body string Request body
---@param callback fun(response: HttpResponse) Callback function
function http.put(url, body, callback) end

---Send a DELETE request
---@param url string The request URL
---@param callback fun(response: HttpResponse) Callback function
function http.delete(url, callback) end

---Send a PATCH request
---@param url string The request URL
---@param body string Request body
---@param callback fun(response: HttpResponse) Callback function
function http.patch(url, body, callback) end

---Send a custom HTTP request with full options
---@param opts RequestOptions The request options
---@param callback fun(response: HttpResponse) Callback function
function http.request(opts, callback) end

---@class RequestOptions
---@field url string Request URL
---@field method string HTTP method (GET/POST/PUT/DELETE/PATCH)
---@field headers table<string, string>? Request headers
---@field body string? Request body
---@field timeout number? Timeout in milliseconds (default: 30000)

lc.http = http

-- ============================================
-- lc.json - JSON encoding/decoding
-- ============================================

---@class lc.json
local json = {}

---Encode a Lua value to a JSON string
---@param value any The Lua value to encode (nil, boolean, number, string, table, array)
---@return string json_string The JSON encoded string
function json.encode(value) end

---Decode a JSON string to a Lua value
---@param json_string string The JSON string to decode
---@return any lua_value The decoded Lua value
function json.decode(json_string) end

lc.json = json

-- ============================================
-- lc.log - Logging
-- ============================================

---Write a log entry to the log file
---@param level string Log level (e.g., "info", "warn", "error", "debug")
---@param format string Format string with {} placeholders
---@vararg any Arguments to format into the message
function lc.log(level, format, ...) end

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
-- lc.interactive - Execute interactive commands
-- ============================================

---Execute a command in interactive mode (with terminal access)
---@param cmd string[] The command and its arguments
---@param on_complete fun(exit_code: number)? Optional callback function called when command exits
function lc.interactive(cmd, on_complete) end

-- ============================================
-- lc.osc52_copy - Copy text to clipboard via OSC 52
-- ============================================

---Copy text to system clipboard using OSC 52 escape sequence
---@param text string The text to copy
function lc.osc52_copy(text) end

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

---@alias EventHook 'EnterPost' | 'HoverPost

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

---Display a notification in bottom-right corner
---@param message string The notification message
function lc.notify(message) end

-- ============================================
-- Filter Mode - Filtering entries
-- ============================================

---Enter filter mode (search/filter mode)
---In filter mode, character input will filter the displayed entries
function api.enter_filter_mode() end

---Exit filter mode and clear the filter
function api.exit_filter_mode() end

---Exit filter mode but keep the current filter applied
function api.accept_filter() end

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
-- lc.tbl_extend - Deep extend table with sources
-- ============================================

---Deep extend target table with values from source tables
---@param target table The target table to extend
---@vararg table Source tables to copy values from
---@return table The extended target table
function lc.tbl_extend(target, ...) end

-- ============================================
-- lc.validate - Validate arguments (vim.validate style)
-- ============================================

---Validate arguments using vim.validate-style API
---Errors are shown via lc.notify instead of throwing
---@param args table<string, {value: any, type: string, optional?: boolean}|any[]> Validation specifications
---@return boolean valid Whether all validations passed
---@example
---lc.validate({
---  name = {name, 'string'},
---  age = {age, 'number'},
---  optional = {optional_value, 'string', true},
---})
function lc.validate(args) end

-- ============================================
-- Type aliases
-- ============================================

---@alias Mode "main"|"input"

---@class PageEntry
---@field key string The unique key for the entry
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
