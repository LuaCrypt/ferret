-- expect: pass
-- category: regressions

local callable = setmetatable({ base = 7 }, {
    __call = function(self, value)
        return self.base + value
    end
})
local total = 0
for i = 1, 6 do
    total = total + callable(i)
end
print("regressions_015", total)
