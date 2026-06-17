-- expect: pass
-- category: goto_labels

local n = 0
::again::
n = n + 1
if n < 3 then goto again end
print('goto_001', n)
