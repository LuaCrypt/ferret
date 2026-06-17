-- expect: pass
-- category: generic_for

local total = 0
for idx, value in ipairs({14, 15, 16}) do
    total = total + idx * value
end
print('generic_014', total)
