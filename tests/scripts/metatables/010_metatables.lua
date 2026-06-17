-- expect: pass
-- category: metatables

local mt = { __mul = function(a, b) return a.v * b.v end }
local a = setmetatable({ v = 6 }, mt)
local b = setmetatable({ v = 7 }, mt)
print('metatables_010', a * b)
