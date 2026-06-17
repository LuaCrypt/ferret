-- expect: pass
-- category: locals_assignments

local x = 1
local y = x + 10
local x = y * 2
local a, b, c = x, nil, y
b = a - c
print('locals_001', x, a, b, c == y)
