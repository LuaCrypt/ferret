-- expect: pass
-- category: metatables

local mt = { __le = function(a, b) return a.v <= b.v end }
local a = setmetatable({ v = 5 }, mt)
local b = setmetatable({ v = 5 }, mt)
print('metatables_008', a <= b)
