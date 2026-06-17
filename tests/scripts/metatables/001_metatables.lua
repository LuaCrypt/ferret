-- expect: pass
-- category: metatables

local mt = { __add = function(a, b) return a.v + b.v end }
local a = setmetatable({ v = 7 }, mt)
local b = setmetatable({ v = 9 }, mt)
print('metatables_001', a + b)
