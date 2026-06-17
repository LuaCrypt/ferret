-- expect: pass
-- category: regressions

local mt = {
    __add = function(left, right)
        return { value = left.value + right.value + 1 }
    end
}
local total = setmetatable({ value = 0 }, mt)
for i = 1, 5 do
    total = total + setmetatable({ value = i }, mt)
end
print("regressions_013", total.value)
