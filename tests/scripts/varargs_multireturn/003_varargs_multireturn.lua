-- expect: pass
-- category: varargs_multireturn

local function first(...)
    return ...
end
local value = first('v3', 'ignored')
print('varargs_003', value)
