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

lc.fs = fs
