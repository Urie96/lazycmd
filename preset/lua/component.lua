---@class SelectOption
---@field value any The value to return when this option is selected
---@field display string|Span|Line|Text The text to display for this option (supports styled widgets)

---@class SelectOptions
---@field prompt? string Optional prompt/title text (defaults to "Select")
---@field options (string|SelectOption)[] The list of options to display

---Show a selection dialog to the user
---The dialog appears centered on screen with a list of options
---Users can navigate with arrow keys (or j/k), type to filter, Enter to select, Esc to cancel
---@param opts SelectOptions Configuration options
---  Can be simple strings: {"Option 1", "Option 2", "Option 3"}
---  Or tables with value/display: {{value = "py", display = "🐍 Python"}, {value = "js", display = "📜 JavaScript"}}
---@param on_selection fun(choice: any) Callback function when user makes a selection
---  - Called with the selected value (the value field from options, or the string itself)
---  - Called with nil if user cancels (Esc)
function lc.select(opts, on_selection) return _lc.select(opts, on_selection) end

---@class ConfirmOptions
---@field title? string Optional title text (defaults to "Confirm")
---@field prompt string The confirmation message to display
---@field on_confirm fun() Callback function when user confirms (Yes)
---@field on_cancel? fun() Callback function when user cancels (No)

---Show a confirmation dialog to the user
---The dialog appears centered on screen with Yes/No buttons
---Users can use Left/Right arrows to select buttons, Enter to confirm selection
---Or use Y/N keys to directly confirm or cancel
---@param opts ConfirmOptions Configuration options
function lc.confirm(opts) return _lc.confirm(opts) end

---Display a notification in bottom-right corner
---@param message string The notification message
function lc.notify(message) return _lc.notify(message) end

---Write a log entry to the log file
---@param level string Log level (e.g., "info", "warn", "error", "debug")
---@param format string Format string with {} placeholders
---@vararg any Arguments to format into the message
function lc.log(level, format, ...) return _lc.log(level, format, ...) end
