function lc.tbl_map(func, t)
  local rettab = {} --- @type table<any,any>
  for k, v in pairs(t) do
    rettab[k] = func(v)
  end
  return rettab
end

---@param self string
---@param sep string
function string.split(self, sep)
  return lc.split(self, sep)
end

unpack = unpack or table.unpack
