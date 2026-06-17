-- expect: pass
-- category: generic_for

local total = 0
for idx, value in ipairs({2, 3, 4}) do
    total = total + idx * value
end
print('generic_002', total)
