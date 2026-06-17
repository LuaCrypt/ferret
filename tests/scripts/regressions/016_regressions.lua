-- expect: pass
-- category: regressions

local function fail_at(limit)
    local total = 0
    for i = 1, limit do
        total = total + i
        if i == 4 then
            error("trace_error_marker")
        end
    end
    return total
end
local ok = pcall(function()
    return fail_at(8)
end)
print("regressions_016", ok)
