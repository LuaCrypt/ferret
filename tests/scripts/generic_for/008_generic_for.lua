-- expect: pass
-- category: generic_for

local total = 0
for idx, value in ipairs({8, 9, 10}) do
    total = total + idx * value
end
print('generic_008', total)
