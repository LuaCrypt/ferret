-- expect: pass
-- category: locals_assignments

local x = 7
local y = x + 10
local x = y * 2
local a, b, c = x, nil, y
b = a - c
print('locals_007', x, a, b, c == y)
