-- expect: pass
-- category: generic_for

local total = 0
for idx, value in ipairs({6, 7, 8}) do
    total = total + idx * value
end
print('generic_006', total)
