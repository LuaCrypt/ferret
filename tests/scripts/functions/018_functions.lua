-- expect: pass
-- category: functions

local t = { base = 18 }
function t:add(v)
    return self.base + v
end
print('functions_018', t:add(23))
