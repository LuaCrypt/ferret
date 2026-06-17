-- expect: pass
-- category: regressions
-- protected-string: REGRESSION_SECRET_005

local function outer(a)
    local secret = 'REGRESSION_SECRET_005'
    return function(b)
        return #secret + a + b
    end
end
local fn = outer(5)
print('regressions_005', fn(6))
