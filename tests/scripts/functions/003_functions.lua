-- expect: pass
-- category: functions

local t = { base = 3 }
function t:add(v)
    return self.base + v
end
print('functions_003', t:add(8))
