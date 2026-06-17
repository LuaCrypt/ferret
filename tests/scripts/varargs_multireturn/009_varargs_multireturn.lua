-- expect: pass
-- category: varargs_multireturn

local function many()
    return 1, nil, 9
end
local a, b, c = many()
print('multireturn_009', a, b == nil, c)
