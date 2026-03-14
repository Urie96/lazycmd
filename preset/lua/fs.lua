---@class FileStat
---@field exists boolean Whether the file/directory exists
---@field is_file boolean Whether it's a file
---@field is_dir boolean Whether it's a directory
---@field is_readable boolean Whether it's readable
---@field is_writable boolean Whether it's writable
---@field is_executable boolean Whether it's executable

---@class TempfileOptions
---@field prefix string? File name prefix (e.g., "lazycmd")
---@field suffix string? File name suffix/extension (e.g., ".log" or "log")
---@field content string? Initial content to write to the file

---@class lc.fs
local fs = {}

---Read directory synchronously
---@param path string The directory path to read
---@return table[] entries List of directory entries
---@return string|nil error Error message if failed
function fs.read_dir_sync(path) return _lc.fs.read_dir_sync(path) end

---Read file content synchronously
---@param path string The file path to read
---@return string content The file content
---@return string|nil error Error message if failed
function fs.read_file_sync(path) return _lc.fs.read_file_sync(path) end

---Write content to file synchronously
---@param path string The file path to write
---@param content string The content to write
---@return boolean success Whether the write succeeded
---@return string|nil error Error message if failed
function fs.write_file_sync(path, content) return _lc.fs.write_file_sync(path, content) end

---Get file/directory statistics synchronously
---@param path string The file or directory path
---@return FileStat stat Statistics about the path
function fs.stat(path) return _lc.fs.stat(path) end

---Create directory and all parent directories if they don't exist (like mkdir -p)
---@param path string The directory path to create
---@return boolean success Whether the creation succeeded
---@return string|nil error Error message if failed
function fs.mkdir(path) return _lc.fs.mkdir(path) end

---Create a temporary file in system temp directory
---@param opts TempfileOptions? Optional settings for filename
---@return string path The path to the created temporary file
---@return string|nil error Error message if failed
---[[
-- Examples:
--   local path = lc.fs.tempfile()                              -- → "/tmp/tmp.a1b2c3d4"
--   local path = lc.fs.tempfile({prefix = "memo"})             -- → "/tmp/memo.a1b2c3d4"
--   local path = lc.fs.tempfile({suffix = ".log"})             -- → "/tmp/tmp.a1b2c3d4.log"
--   local path = lc.fs.tempfile({prefix = "memo", suffix = ".md"}) -- → "/tmp/memo.a1b2c3d4.md"
--   local path = lc.fs.tempfile({content = "hello world"})    -- → "/tmp/tmp.a1b2c3d4" with content
--]]
function fs.tempfile(opts) return _lc.fs.tempfile(opts) end

---Remove a file or directory
---@param path string The file or directory path to remove (directories are removed recursively)
---@return boolean success Whether the removal succeeded
---@return string|nil error Error message if failed
---[[
-- Examples:
--   local ok, err = lc.fs.remove("/tmp/myfile.txt")     -- Remove file
--   local ok, err = lc.fs.remove("/tmp/mydir")          -- Remove directory recursively
--   if ok then
--     print("Removed successfully")
--   else
--     print("Error: " .. err)
--   end
--]]
function fs.remove(path) return _lc.fs.remove(path) end

lc.fs = fs
