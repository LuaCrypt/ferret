-- expect: pass
-- category: basics

local a = (8 << 2) | 3
local b = (a & 15) ~ 1
print('basics_008', a, b, ~0)
