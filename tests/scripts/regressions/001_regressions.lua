-- expect: pass
-- category: regressions
-- protected-string: REGRESSION_SECRET_001

local function outer(a)
    local secret = 'REGRESSION_SECRET_001'
    return function(b)
        return #secret + a + b
    end
end
local fn = outer(1)
print('regressions_001', fn(2))
