---@class lc.style
local style = {}

---@class Text
---A TUI Text widget
---@field append fun(self: Text, line: Line|Span|string) Append a line to the text (modifies in place)

---@class Span
---A TUI Span widget
---@field fg fun(self: Span, color: string): Span Set foreground color (modifies in place and returns self)
---@field bg fun(self: Span, color: string): Span Set background color (modifies in place and returns self)

---@class Line
---A TUI Line widget containing multiple Spans
---@field fg fun(self: Line, color: string): Line Set foreground color (modifies in place and returns self)
---@field bg fun(self: Line, color: string): Line Set background color (modifies in place and returns self)

---Create a Span from a string
---@param s string The string into a Line
---@return Span
function style.span(args) return _lc.style.span(args) end

---Create a Line from a table of Spans or Strings
---@param args (Span|string)[] The Spans or Strings to combine into a Line
---@return Line A Line widget containing the combined Spans
function style.line(args) return _lc.style.line(args) end

---Create a Text from a table of Lines, Spans, or Strings
---@param args (Line|Span|string)[] The Lines, Spans, or Strings to combine into a Text
---@return Text A Text widget containing the combined content
function style.text(args) return _lc.style.text(args) end

---Highlight code with syntax highlighting
---@param code string The code to highlight
---@param language string The programming language name (e.g., "javascript", "python", "rust", "lua")
---@return Text A Text widget with syntax-highlighted code
function style.highlight(code, language) return _lc.style.highlight(code, language) end

---Align columns in a 1D array of Lines, modifying them in place
---@param lines Line[] A 1D array of Lines, where each Line contains multiple Spans representing columns
function style.align_columns(lines) return _lc.style.align_columns(lines) end

lc.style = style
