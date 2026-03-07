local M = {}

function M.setup()
  lc.keymap.set('main', 'q', function() lc.cmd 'quit' end)

  lc.keymap.set('main', '<down>', function() lc.cmd 'scroll_by 1' end)

  lc.keymap.set('main', '<up>', function() lc.cmd 'scroll_by -1' end)
end

function M.list(path, cb)
  -- Create a 1D array of Lines with different content lengths
  local data = {
    -- Header line
    lc.style.line {
      lc.style.span('Name'):fg 'blue',
      lc.style.span '  ',
      lc.style.span('Value'):fg 'green',
      lc.style.span '  ',
      lc.style.span('Description'):fg 'yellow',
    },
    -- Data lines
    lc.style.line {
      lc.style.span 'foo',
      lc.style.span '  ',
      lc.style.span '12345',
      lc.style.span '  ',
      lc.style.span 'A short description',
    },
    lc.style.line {
      lc.style.span 'bar',
      lc.style.span '  ',
      lc.style.span '678',
      lc.style.span '  ',
      lc.style.span 'Another description here',
    },
    lc.style.line {
      lc.style.span 'bazqux',
      lc.style.span '  ',
      lc.style.span '1234567890',
      lc.style.span '  ',
      lc.style.span 'Long description text example',
    },
    lc.style.line {
      lc.style.span 'x',
      lc.style.span '  ',
      lc.style.span '42',
      lc.style.span '  ',
      lc.style.span 'Short',
    },
  }

  -- Print before alignment for debugging
  -- lc.log('info', 'Before alignment:')
  -- for i, line in ipairs(data) do
  --   lc.log('info', 'Line ' .. i)
  -- end

  -- Align columns - this modifies the Spans in place
  lc.style.align_columns(data)

  -- Print after alignment for debugging
  -- lc.log('info', 'After alignment:')
  -- for i, line in ipairs(data) do
  --   lc.log('info', 'Line ' .. i)
  -- end

  -- Convert to entries
  local entries = {}
  for i, line in ipairs(data) do
    table.insert(entries, {
      key = tostring(i),
      display = line,
    })
  end

  cb(entries)
end

function M.preview(entry, cb) end

return M
