-- expect: pass
-- category: varargs_multireturn

local function first(...)
    return ...
end
local value = first('v4', 'ignored')
print('varargs_004', value)
