-- expect: pass
-- category: varargs_multireturn

local function many()
    return 1, nil, 14
end
local a, b, c = many()
print('multireturn_014', a, b == nil, c)
