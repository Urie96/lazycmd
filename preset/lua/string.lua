---Set text color for display
---@param color string Color name (e.g., "blue", "red", "green")
---@return Span A colored span widget
function string:fg(color) return _lc.style.span(self):fg(color) end

---Parse ANSI escape sequences into a TUI Text widget
---@return Text A Text widget with parsed ANSI codes
function string:ansi() return _lc.string.ansi(self) end

---Split string by separator
---@param sep string The separator
---@return string[] The split parts
function string:split(sep) return _lc.split(self, sep) end

---Trim leading and trailing whitespace from a string
---@return string trimmed The trimmed string
function string:trim() return string.match(self, '^%s*(.*%S)' or '') end
