function lc.tbl_map(func, t)
  local rettab = {} --- @type table<any,any>
  for k, v in pairs(t) do
    rettab[k] = func(v)
  end
  return rettab
end

---@param self string
---@param sep string
function string.split(self, sep) return lc.split(self, sep) end

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

---Validate arguments using vim.validate-style API
---Errors are shown via lc.notify instead of throwing
---@param args table<string, {value: any, type: string, optional?: boolean}|any[]>
---Example: lc.validate({ name = {name, 'string'}, age = {age, 'number'} })
function lc.validate(args)
  local function validate_one(name, value, expected_type, optional)
    if optional and (value == nil or value == vim.NIL) then
      return true
    end

    local actual_type = type(value)

    -- Handle table with expected type
    if expected_type == 'table' and actual_type == 'table' then
      return true
    end

    -- Handle function type
    if expected_type == 'function' and actual_type == 'function' then
      return true
    end

    -- Handle basic types
    if actual_type ~= expected_type then
      lc.notify(string.format('Validation failed: %s expected %s, got %s', name, expected_type, actual_type))
      return false
    end

    return true
  end

  local all_valid = true
  for name, spec in pairs(args) do
    local value, expected_type, optional
    local spec_type = type(spec)

    if spec_type == 'table' then
      -- Format: { value, type_name, optional? }
      value = spec[1]
      expected_type = spec[2]
      optional = spec[3]
    elseif spec_type == 'string' then
      -- Format: { value, type_name } with spec as value and args[name] as type
      value = spec
      expected_type = args[name .. '_type']
      optional = args[name .. '_optional']
    else
      lc.notify(string.format('Validation failed: invalid spec for %s', name))
      all_valid = false
      goto continue
    end

    if not expected_type then
      lc.notify(string.format('Validation failed: missing type for %s', name))
      all_valid = false
      goto continue
    end

    if not validate_one(name, value, expected_type, optional) then
      all_valid = false
    end

    ::continue::
  end

  return all_valid
end
