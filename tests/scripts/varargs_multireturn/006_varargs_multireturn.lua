-- expect: pass
-- category: varargs_multireturn

local function first(...)
    return ...
end
local value = first('v6', 'ignored')
print('varargs_006', value)
