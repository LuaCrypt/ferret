-- expect: pass
-- category: varargs_multireturn

local function first(...)
    return ...
end
local value = first('v8', 'ignored')
print('varargs_008', value)
