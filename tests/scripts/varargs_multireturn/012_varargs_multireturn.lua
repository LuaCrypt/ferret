-- expect: pass
-- category: varargs_multireturn

local function many()
    return 1, nil, 12
end
local a, b, c = many()
print('multireturn_012', a, b == nil, c)
