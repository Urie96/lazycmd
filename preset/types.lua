-- Type definitions for lazycmd Lua API
-- This file provides type hints for Lua LSP/EmmyLua

-- ============================================
-- Global: lc
-- ============================================

---@class lc
---@field style lc.style Style utilities for creating widgets
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
---@param widget string|Text|Line The widget to display in the preview panel
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

---Append a hook callback to be called before reload command
---@param callback fun() The callback function to execute before reload
function api.append_hook_pre_reload(callback) end

lc.api = api

-- ============================================
-- lc.fs - File system operations
-- ============================================

---@class FileStat
---@field exists boolean Whether the file/directory exists
---@field is_file boolean Whether it's a file
---@field is_dir boolean Whether it's a directory
---@field is_readable boolean Whether it's readable
---@field is_writable boolean Whether it's writable
---@field is_executable boolean Whether it's executable

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

---Get file/directory statistics synchronously
---@param path string The file or directory path
---@return FileStat stat Statistics about the path
function fs.stat(path) end

---Create directory and all parent directories if they don't exist (like mkdir -p)
---@param path string The directory path to create
---@return boolean success Whether the creation succeeded
---@return string|nil error Error message if failed
function fs.mkdir(path) end

lc.fs = fs

-- ============================================
-- lc.cache - Persistent caching
-- ============================================

---@class CacheOptions
---@field ttl number? Time-to-live in seconds (optional)

---@class lc.cache
local cache = {}

---Get a value from cache
---@param key string The cache key
---@return any value The cached value, or nil if not found or expired
function cache.get(key) end

---Set a value in cache
---@param key string The cache key
---@param value any The value to cache (nil, boolean, number, string, table, array)
---@param opts CacheOptions? Optional options (e.g., {ttl = 3600} for 1 hour TTL)
function cache.set(key, value, opts) end

---Delete a value from cache
---@param key string The cache key to delete
function cache.delete(key) end

---Clear all cached values
function cache.clear() end

lc.cache = cache

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
-- lc.time - Time and timestamp operations
-- ============================================

---@class lc.time
local time = {}

---Get the current Unix timestamp
---@return number timestamp The current Unix timestamp (seconds since epoch)
function time.now() end

---Parse an ISO 8601 datetime string and return Unix timestamp
---@param time_str string The time string to parse (e.g., "2023-12-25T15:30:45Z", "2023-12-25T15:30:45+08:00")
---@return number timestamp The Unix timestamp (seconds since epoch)
function time.parse(time_str) end

---Format a Unix timestamp to an ISO 8601 string (or custom format)
---@param timestamp number The Unix timestamp (seconds since epoch)
---@param format_str string? Optional format string:
--- - "compact" - Compact format: HH:MM for today, MM-DD for this year, YYYY-MM for older dates
--- - "%Y-%m-%d" or any chrono format string
--- - Defaults to ISO 8601 (e.g., "2023-12-25T15:30:45Z")
---@return string formatted The formatted datetime string
function time.format(timestamp, format_str) end

lc.time = time

-- ============================================
-- lc.system - Execute external commands
-- ============================================

---@class CommandOutput
---@field code number Exit code
---@field stdout string Standard output
---@field stderr string Standard error

---@class lc.system
local system = {}

---Execute an external command asynchronously
---Usage: lc.system({"cmd", "arg1", "arg2"}, callback)
---@param cmd string[] The command and its arguments
---@param callback fun(output: CommandOutput) Callback function called on completion
function system(cmd, callback) end

---Check if a command is executable (synchronous)
---@param cmd string The command name to check
---@return boolean executable Whether the command exists and is executable
function system.executable(cmd) end

---Open a file using the system's default application
---Cross-platform support: uses 'open' on macOS, 'xdg-open' on Linux, 'start' on Windows
---@param file_path string The path to the file to open
function system.open(file_path) end

lc.system = system

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

---@alias EventHook 'EnterPost' | 'HoverPost'

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
-- lc.style - Style utilities
-- ============================================

---@class lc.style
local style = {}

---Create a Line from a table of Spans or Strings
---@param args (Span|string)[] The Spans or Strings to combine into a Line
---@return Line A Line widget containing the combined Spans
function lc.style.line(args) end

---Create a Text from a table of Lines, Spans, or Strings
---@param args (Line|Span|string)[] The Lines, Spans, or Strings to combine into a Text
---@return Text A Text widget containing the combined content
function lc.style.text(args) end

-- ============================================
-- lc.config - Configure plugins
-- ============================================

---Configure plugins and settings
---@param opt {default_plugin?: string, plugins?: table[]} Configuration options
---| {default_plugin: string, plugins: {name: string, config: fun()}[]} Plugin configuration
function lc.config(opt) end

-- ============================================
-- lc.equals - Compare two values for equality
-- ============================================

---Compare two values for deep equality (including tables)
---@param o1 any First value to compare
---@param o2 any Second value to compare
---@param ignore_mt boolean? Whether to ignore metatables (default: false)
---@return boolean equal Whether the values are equal
function lc.equals(o1, o2, ignore_mt) end

-- ============================================
-- lc.trim - Trim whitespace from string
-- ============================================

---Trim leading and trailing whitespace from a string
---@param s string The string to trim
---@return string trimmed The trimmed string
function lc.trim(s) end

-- ============================================
-- Type aliases
-- ============================================

---@alias Mode "main"|"input"

---@class PageEntry
---@field key string The unique key for the entry
---@field display? string|Span The display text or Span widget
---@field pid? number Process ID (for process plugin)
---@field [string] any Additional custom fields

---@class Text
---A TUI Text widget

---@class Span
---A TUI Span widget

---Set foreground color for the Span (modifies in place and returns self)
---@param color string Color name (e.g., "blue", "red", "green")
---@return Span self Returns self for method chaining
function Span:fg(color) end

---Set background color for the Span (modifies in place and returns self)
---@param color string Color name (e.g., "blue", "red", "green")
---@return Span self Returns self for method chaining
function Span:bg(color) end

---@class Line
---A TUI Line widget containing multiple Spans

---Set foreground color for the Line (modifies in place and returns self)
---@param color string Color name (e.g., "blue", "red", "green")
---@return Line self Returns self for method chaining
function Line:fg(color) end

---Set background color for the Line (modifies in place and returns self)
---@param color string Color name (e.g., "blue", "red", "green")
---@return Line self Returns self for method chaining
function Line:bg(color) end

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
