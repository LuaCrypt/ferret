-- expect: pass
-- category: metatables

local t = setmetatable({ 1, 2, 3 }, { __len = function() return 99 end })
print('metatables_004', #t)
