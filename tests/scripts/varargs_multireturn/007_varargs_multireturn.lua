-- expect: pass
-- category: varargs_multireturn

local function first(...)
    return ...
end
local value = first('v7', 'ignored')
print('varargs_007', value)
