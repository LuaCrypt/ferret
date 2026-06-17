-- expect: pass
-- category: generic_for

local total = 0
for idx, value in ipairs({10, 11, 12}) do
    total = total + idx * value
end
print('generic_010', total)
