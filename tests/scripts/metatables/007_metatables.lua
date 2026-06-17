-- expect: pass
-- category: metatables

local mt = { __lt = function(a, b) return a.v < b.v end }
local a = setmetatable({ v = 3 }, mt)
local b = setmetatable({ v = 5 }, mt)
print('metatables_007', a < b)
