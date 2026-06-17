-- expect: pass
-- category: functions

local function fact(n)
    if n <= 1 then return 1 end
    return n * fact(n - 1)
end
print('functions_001', fact(4))
