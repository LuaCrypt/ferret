-- expect: pass
-- category: functions

local t = { base = 9 }
function t:add(v)
    return self.base + v
end
print('functions_009', t:add(14))
