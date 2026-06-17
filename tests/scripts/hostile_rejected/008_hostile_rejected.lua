-- expect: reject
-- category: hostile_rejected

local f = load('return 1')
print(f())
