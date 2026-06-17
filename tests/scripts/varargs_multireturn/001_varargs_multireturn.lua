-- expect: pass
-- category: varargs_multireturn

local function first(...)
    return ...
end
local value = first('v1', 'ignored')
print('varargs_001', value)
