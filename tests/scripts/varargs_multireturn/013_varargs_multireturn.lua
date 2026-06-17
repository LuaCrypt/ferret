-- expect: pass
-- category: varargs_multireturn

local function many()
    return 1, nil, 13
end
local a, b, c = many()
print('multireturn_013', a, b == nil, c)
