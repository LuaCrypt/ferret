-- expect: pass
-- category: regressions

local function choose(flag)
    if flag then return 'yes' end
    return 'no'
end
print('regressions_011', choose(false))
