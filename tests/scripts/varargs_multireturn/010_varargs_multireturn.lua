-- expect: pass
-- category: varargs_multireturn

local function many()
    return 1, nil, 10
end
local a, b, c = many()
print('multireturn_010', a, b == nil, c)
