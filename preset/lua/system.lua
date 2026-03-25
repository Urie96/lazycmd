---@class CommandOutput
---@field code number Exit code
---@field stdout string Standard output
---@field stderr string Standard error

---@class SystemOptions
---@field stdin string? Optional standard input to provide to the command
---@field env table<string, string>? Optional environment variables to set for the command
---@field callback fun(output: CommandOutput)? Callback function called on completion

---@class lc.system
local system = {}

---Execute an external command asynchronously (Lua wrapper)
---This wrapper provides multiple convenient call formats:
---Usage 1: lc.system.exec({cmd, callback})
---Usage 2: lc.system.exec(cmd, callback)
---Usage 3: lc.system.exec(cmd, opts, callback)
---
---The wrapper calls lc.system._exec internally after parameter processing
---@param cmd table The arguments table or command array
---@param opts_or_callback SystemOptions|fun(output: CommandOutput)? Options table or callback function
---@param callback fun(output: CommandOutput)? Callback function
function system.exec(cmd, opts_or_callback, callback)
  -- Parse arguments:
  -- lc.system.exec(cmd, callback)
  -- lc.system.exec(cmd, opts, callback)

  local args_table = { cmd = cmd }

  if type(opts_or_callback) == 'function' then
    -- lc.system.exec(cmd, callback)
    args_table.callback = opts_or_callback
  elseif type(opts_or_callback) == 'table' then
    -- lc.system.exec(cmd, opts, callback)
    if opts_or_callback.stdin ~= nil then args_table.stdin = opts_or_callback.stdin end
    if opts_or_callback.env ~= nil then args_table.env = opts_or_callback.env end
    if type(callback) == 'function' then
      args_table.callback = callback
    elseif opts_or_callback.callback ~= nil then
      args_table.callback = opts_or_callback.callback
    else
      error 'Callback function is required when providing options'
    end
  else
    error 'Callback function is required'
  end

  -- Call the Rust implementation
  _lc.system.exec(args_table)
end

---Check if a command is executable (synchronous)
---@param cmd string The command name to check
---@return boolean executable Whether the command exists and is executable
function system.executable(cmd) return _lc.system.executable(cmd) end

---Spawn a detached background process without waiting for completion.
---@param cmd string[] The command and its arguments
function system.spawn(cmd) return _lc.system.spawn({ cmd = cmd }) end

---Send a request over a Unix domain socket and receive one line of response.
---@param opts {path: string, message: string}
---@param callback fun(response: {success: boolean, body: string, error: string?})
function system.socket_request(opts, callback)
  opts.callback = callback
  return _lc.system.socket_request(opts)
end

---Send a request over a Unix domain socket synchronously and receive one line of response.
---@param opts {path: string, message: string}
---@return {success: boolean, body: string, error: string?}
function system.socket_request_sync(opts) return _lc.system.socket_request_sync(opts) end

---Open a file using the system's default application
---Cross-platform support: uses 'open' on macOS, 'xdg-open' on Linux, 'start' on Windows
---@param file_path string The path to the file to open
function system.open(file_path) return _lc.system.open(file_path) end

lc.system = system

-- Set metatable on lc.system to handle multiple argument formats
setmetatable(lc.system, {
  __call = function(self, cmd, opts_or_callback, callback) lc.system.exec(cmd, opts_or_callback, callback) end,
})
