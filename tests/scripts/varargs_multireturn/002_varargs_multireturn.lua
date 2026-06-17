-- expect: pass
-- category: varargs_multireturn

local function first(...)
    return ...
end
local value = first('v2', 'ignored')
print('varargs_002', value)
