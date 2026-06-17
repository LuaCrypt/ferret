-- expect: pass
-- category: generic_for

local total = 0
for idx, value in ipairs({12, 13, 14}) do
    total = total + idx * value
end
print('generic_012', total)
