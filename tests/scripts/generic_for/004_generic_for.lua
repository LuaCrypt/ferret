-- expect: pass
-- category: generic_for

local total = 0
for idx, value in ipairs({4, 5, 6}) do
    total = total + idx * value
end
print('generic_004', total)
