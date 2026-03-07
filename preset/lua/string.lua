---Set text color for display
---@param s string content
---@param color string Color name (e.g., "blue", "red", "green")
---@return Span A colored span widget
function string.fg(s, color) return _lc.style.span(s):fg(color) end

---Parse ANSI escape sequences into a TUI Text widget
---@param s string content
---@return Text A Text widget with parsed ANSI codes
function string.ansi(s) return _lc.string.ansi(s) end

---Split string by separator
---@param s string content
---@param sep string The separator
---@return string[] The split parts
function string.split(s, sep) return _lc.split(s, sep) end

---Trim leading and trailing whitespace from a string
---@param s string content
---@return string trimmed The trimmed string
function string.trim(s) return string.match(s, '^%s*(.*%S)' or '') end
