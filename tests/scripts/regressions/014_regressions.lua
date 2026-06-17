-- expect: pass
-- category: regressions

local seen = {}
local t = setmetatable({}, {
    __index = function(_, key)
        return #key
    end,
    __newindex = function(_, key, value)
        seen[key] = value * 2
    end
})
for i = 1, 4 do
    t["k" .. i] = t.missing + i
end
print("regressions_014", seen.k1, seen.k4)
