-- expect: pass
-- category: varargs_multireturn

local function first(...)
    return ...
end
local value = first('v5', 'ignored')
print('varargs_005', value)
