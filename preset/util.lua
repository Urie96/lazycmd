local lc = lc or {}

function lc.tbl_map(func, t)
  local rettab = {} --- @type table<any,any>
  for k, v in pairs(t) do
    rettab[k] = func(v)
  end
  return rettab
end

function lc.tbl_extend(target, ...)
  if type(target) ~= 'table' then error 'tbl_extend_deep: target must be a table' end

  local sources = { ... }

  local function deep_copy(value)
    if type(value) ~= 'table' then return value end

    local copy = {}
    for k, v in pairs(value) do
      copy[deep_copy(k)] = deep_copy(v)
    end
    return copy
  end

  for _, source in ipairs(sources) do
    if type(source) ~= 'table' then error 'tbl_extend_deep: all sources must be tables' end

    for key, value in pairs(source) do
      target[key] = deep_copy(value)
    end
  end

  return target
end

unpack = unpack or table.unpack

---@param self string
---@param sep string
function string.split(self, sep) return lc.split(self, sep) end

---@param o1 any|table First object to compare
---@param o2 any|table Second object to compare
---@param ignore_mt boolean|nil True to ignore metatables (a recursive function to tests tables inside tables)
local function equals(o1, o2, ignore_mt)
  if o1 == o2 then return true end
  local o1Type = type(o1)
  local o2Type = type(o2)
  if o1Type ~= o2Type then return false end
  if o1Type ~= 'table' then return false end

  if not ignore_mt then
    local mt1 = getmetatable(o1)
    if mt1 and mt1.__eq then
      --compare using built in method
      return o1 == o2
    end
  end

  local keySet = {}

  for key1, value1 in pairs(o1) do
    local value2 = o2[key1]
    if value2 == nil or equals(value1, value2, ignore_mt) == false then return false end
    keySet[key1] = true
  end

  for key2, _ in pairs(o2) do
    if not keySet[key2] then return false end
  end
  return true
end

lc.equals = equals

function lc.trim(s) return s and s:match '^%s*(.*%S)' or '' end
