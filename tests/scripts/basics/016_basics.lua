-- expect: pass
-- category: basics

local a = nil
local b = false
local c = 'x'
print('basics_016', a == nil, b or c, not b)
